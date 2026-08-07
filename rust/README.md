# Flappy Bird — WebAssembly

A rewrite of the original Go + SDL2 version, targeting WebAssembly via
`wasm-bindgen` and the canvas 2D API. Runs in the browser; served from
Cloudflare Workers as static assets.

The `master` branch keeps the original Go/SDL2 source. This branch replaces the
rendering layer only — the game is still a bird, some pipes, and a scrolling
ground.

## Why a rewrite and not a recompile

The Go version used `github.com/veandco/go-sdl2`, which is cgo bindings to
native SDL2. Go's wasm target (`GOOS=js GOARCH=wasm`) does not support cgo, so
there is no build configuration that turns the original source into a `.wasm`.
The Emscripten path that usually rescues SDL2 projects is a C/C++ toolchain and
does not accept Go either.

About 80 of the original 519 lines were actual game logic; the rest was SDL
setup and teardown. That logic carried over directly.

## Build

Requires the Rust toolchain, the wasm target, and `wasm-bindgen-cli`:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
# optional, shaves ~10% off the module:
#   https://github.com/WebAssembly/binaryen
```

Then:

```sh
./build.sh           # release build into static/pkg/
./build.sh --dev     # faster, unoptimized
```

Output size:

| file | raw | gzipped |
| --- | --- | --- |
| `flappy_bg.wasm` | 56 KB | 26 KB |
| `flappy.js` | 22 KB | — |

## Tests

The game logic (state machine, physics, collision, scoring) lives in
`src/game.rs`, kept free of any DOM access, so it is unit-tested on the host
target with no browser:

```sh
cargo test
```

The DOM-facing modules (`app`, `render`) are compiled only for `wasm32` and are
`cfg`'d out on the host, so the test build stays lightweight.

## Run locally

```sh
./build.sh
cd static && python3 -m http.server 8080
```

Then open <http://localhost:8080>. A plain file:// open will not work — wasm
modules have to be fetched over http(s).

Or with the Workers runtime, which also exercises `src/worker.js`:

```sh
npx wrangler dev
```

## Deploy

```sh
./build.sh
npx wrangler deploy
```

`static/` is uploaded as the asset store. `src/worker.js` sits in front of it to
force `application/wasm` on the module (without it the browser drops from
streaming instantiation to buffering), set cache headers, and redirect a bare
mount path to its trailing-slash form.

To attach it to a domain, a subdomain such as `flappy.example.com` is the
simplest option — set it up as a custom domain on the Worker. Mounting at a
path (`example.com/flappy*`) also works via a route, since more specific routes
take precedence over a catch-all, but the trailing-slash redirect in the Worker
matters in that case.

## Layout

```
src/lib.rs        game + renderer (the whole port)
src/worker.js     asset serving, MIME and cache headers
static/index.html host page
static/assets/    sprite sheet and pipe texture
static/pkg/       build output — regenerate, don't edit
build.sh          cargo -> wasm-bindgen -> wasm-opt
```

## Notes on the port

Sprite coordinates were re-measured from the alpha channel of
`FlappySprite.png` rather than carried over; several of the originals were
slightly off.

- Bird: three 17x12 frames at x = 3, 31, 59, y = 491. The original used a 20x20
  box on the same 28px stride, which picked up padding.
- Ground: the 168x56 striped base strip at (292, 0). The original used a flat
  green slice of the background, so the scroll was invisible.
- Pipe: `pipe.png` is 52x320 — a 24px cap at full width, then a body inset 2px
  per side. The original stretched the whole texture into an arbitrary-height
  rect, squashing the cap. The cap is now fixed height with only the body
  stretched, and the top pipe is the same sprite mirrored by a canvas transform.

Gameplay changes from the original, which had free WASD movement and the
`Jump()` impulse wired up but never integrated into position:

- Gravity and a flap impulse, with rotation tracking velocity.
- AABB collision with an inset hitbox so near misses feel fair.
- The ceiling clamps instead of killing, matching the original game.
- A short lockout after death so the killing tap doesn't instantly restart.
- Score persisted to `localStorage`.

Physics run on a fixed 120 Hz timestep with an accumulator. Frame delta is
clamped so a backgrounded tab doesn't fast-forward the world on return.

## Known gaps

- Score uses `fillText`, not the atlas digits. The atlas has a clean 12x18 grid
  for 2–9 at x = 292 + 14i, y = 160 and 184, but 0 and 1 are not in that grid.
- No sound.
- The sprites are from the original Flappy Bird and are not licensed for
  redistribution.
