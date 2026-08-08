# Flappy Bird

Two implementations of the same game live in this repo:

| Directory | Version | Renderer | Runs on |
| --- | --- | --- | --- |
| [`go/`](go/) | Original | SDL2 (cgo to native SDL2) | Desktop |
| [`rust/`](rust/) | Port | canvas 2D (Rust to WebAssembly) | Browser |

Both play the same: gravity, a flap impulse, and hit detection against the
pipes and the ground. The `rust/` version is the one that gets deployed, as a
Cloudflare Worker serving the wasm module and its assets.

## `go/`: original SDL2 version

```sh
cd go
go run .          # requires the SDL2, SDL2_image and SDL2_ttf dev libraries
```

Uses `github.com/veandco/go-sdl2` (cgo bindings to native SDL2). See
[`go/README.md`](go/README.md).

## `rust/`: WebAssembly port

Rewrites the rendering layer against the canvas 2D API and compiles to
WebAssembly with `wasm-bindgen`. Go's wasm target can't compile the original
because `go-sdl2` is cgo, so this is a rewrite of the rendering layer rather
than a recompile; the game logic carried over.

```sh
cd rust
./build.sh                        # release build into static/pkg/
cd static && python3 -m http.server 8080   # then open http://localhost:8080
```

Deploy (Cloudflare Workers):

```sh
cd rust
./build.sh
npx wrangler deploy
```

Full details, including build requirements, the port notes, and known gaps,
are in [`rust/README.md`](rust/README.md).
