# Flappy Bird — original (Go + SDL2)

![flappy](http://d.pr/i/UjwyY+)

Uses SDL2 bindings (`github.com/veandco/go-sdl2`, cgo → native SDL2).

Use WASD to move.

## Install

Requires SDL2 installed on the system. From this directory:

```sh
go run *.go
```

## TODO

- Hit detection
- Jumping

> Both TODOs are implemented in the WebAssembly port under [`../rust/`](../rust/),
> which also adds gravity-based flapping. This original is kept as-is.
