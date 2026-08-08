# Flappy Bird, original (Go + SDL2)

![flappy](http://d.pr/i/UjwyY+)

Uses SDL2 bindings (`github.com/veandco/go-sdl2`, cgo to native SDL2).

## Controls

- Space, W, up arrow, or left click: flap. Also starts a run, and restarts after a death.
- Escape: quit.

Gravity pulls the bird down each frame; hitting a pipe or the ground ends
the run. The ceiling only blocks, it does not kill.

## Install

Requires the SDL2, SDL2_image and SDL2_ttf development libraries on the
system. From this directory:

```sh
go run .
```

A gravity-based WebAssembly port of the same game lives under
[`../rust/`](../rust/).
