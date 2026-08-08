package main

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/veandco/go-sdl2/sdl"
	"github.com/veandco/go-sdl2/ttf"
)

const (
	winTitle  = "Go SDL2 Flappybird"
	winWidth  = 480
	winHeight = 800
)

// The world advances one fixed step of dt seconds per rendered frame.
const (
	frameDelayMS = 16
	dt           = float64(frameDelayMS) / 1000
)

// assetDir is resolved relative to the working directory; run from go/.
const assetDir = "assets"

type gameState int

const (
	stateReady gameState = iota
	statePlaying
	stateDead
)

// stateLabel holds the overlay text per state; states without an entry
// draw no text.
var stateLabel = map[gameState]string{
	stateReady: "READY",
	stateDead:  "GAME OVER",
}

// Engine owns the SDL objects and the game state machine.
type Engine struct {
	state     gameState
	window    *sdl.Window
	renderer  *sdl.Renderer
	scene     *Scene
	font      *ttf.Font
	stateText map[gameState]*text
	running   bool
}

// text is a pre-rendered label texture.
type text struct {
	w, h    int32
	texture *sdl.Texture
}

func NewEngine() *Engine {
	return &Engine{running: true}
}

// Init initializes SDL and creates the window, renderer and scene.
func (e *Engine) Init() error {
	if err := sdl.Init(sdl.INIT_EVERYTHING); err != nil {
		return fmt.Errorf("could not initialize SDL: %v", err)
	}

	if err := ttf.Init(); err != nil {
		return fmt.Errorf("could not initialize TTF: %v", err)
	}

	var err error
	e.window, err = sdl.CreateWindow(winTitle, sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED, winWidth, winHeight, sdl.WINDOW_SHOWN)
	if err != nil {
		return fmt.Errorf("could not create window: %v", err)
	}

	e.renderer, err = sdl.CreateRenderer(e.window, -1, sdl.RENDERER_ACCELERATED)
	if err != nil {
		return fmt.Errorf("could not create renderer: %v", err)
	}

	e.scene, err = NewScene(e.renderer)
	return err
}

// Destroy releases SDL resources; it tolerates a partially failed Init.
func (e *Engine) Destroy() {
	for _, t := range e.stateText {
		if err := t.texture.Destroy(); err != nil {
			sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "destroy text: %s", err)
		}
	}
	if e.font != nil {
		e.font.Close()
	}
	if e.scene != nil {
		e.scene.Destroy()
	}
	if e.renderer != nil {
		if err := e.renderer.Destroy(); err != nil {
			sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "destroy renderer: %s", err)
		}
	}
	if e.window != nil {
		if err := e.window.Destroy(); err != nil {
			sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "destroy window: %s", err)
		}
	}

	ttf.Quit()
	sdl.Quit()
}

// Quit stops the main loop.
func (e *Engine) Quit() {
	e.running = false
}

// Running reports whether the main loop should keep going.
func (e *Engine) Running() bool {
	return e.running
}

// Load opens the font and pre-renders the state labels.
func (e *Engine) Load() error {
	font, err := ttf.OpenFont(filepath.Join(assetDir, "fonts", "Flappy.ttf"), 24)
	if err != nil {
		return fmt.Errorf("could not open font: %v", err)
	}
	e.font = font

	e.stateText = make(map[gameState]*text, len(stateLabel))
	white := sdl.Color{R: 255, G: 255, B: 255, A: 255}
	for state, label := range stateLabel {
		texture, err := e.renderText(label, white)
		if err != nil {
			return fmt.Errorf("could not render %q: %v", label, err)
		}
		_, _, w, h, err := texture.Query()
		if err != nil {
			return fmt.Errorf("could not query %q texture: %v", label, err)
		}
		e.stateText[state] = &text{w: w, h: h, texture: texture}
	}
	return nil
}

// renderText renders a string to a texture using the loaded font.
func (e *Engine) renderText(s string, color sdl.Color) (*sdl.Texture, error) {
	surface, err := e.font.RenderUTF8Blended(s, color)
	if err != nil {
		return nil, err
	}
	defer surface.Free()

	return e.renderer.CreateTextureFromSurface(surface)
}

// flap is the single input action: it starts a run, flaps mid-run and
// restarts after a death.
func (e *Engine) flap() {
	switch e.state {
	case stateReady:
		e.state = statePlaying
		e.scene.bird.Flap()
	case statePlaying:
		e.scene.bird.Flap()
	case stateDead:
		e.scene.Reset()
		e.state = stateReady
	}
}

func (e *Engine) handleEvents() {
	for event := sdl.PollEvent(); event != nil; event = sdl.PollEvent() {
		switch t := event.(type) {
		case *sdl.QuitEvent:
			e.Quit()

		case *sdl.MouseButtonEvent:
			if t.Type == sdl.MOUSEBUTTONDOWN && t.Button == sdl.BUTTON_LEFT {
				e.flap()
			}

		case *sdl.KeyboardEvent:
			if t.Type != sdl.KEYDOWN || t.Repeat != 0 {
				break
			}
			switch t.Keysym.Scancode {
			case sdl.SCANCODE_SPACE, sdl.SCANCODE_W, sdl.SCANCODE_UP:
				e.flap()
			case sdl.SCANCODE_ESCAPE, sdl.SCANCODE_AC_BACK:
				e.Quit()
			}
		}
	}
}

// update advances the world by one frame according to the current state.
func (e *Engine) update() {
	switch e.state {
	case stateReady:
		e.scene.bird.Animate()
	case statePlaying:
		e.scene.Update(dt)
		if e.scene.Collides() {
			e.state = stateDead
		}
	case stateDead:
		e.scene.bird.Fall(dt)
	}
}

// paint draws the scene, overlays the state label and presents the frame.
func (e *Engine) paint() error {
	if err := e.scene.Paint(e.renderer); err != nil {
		return err
	}

	if t, ok := e.stateText[e.state]; ok {
		dst := &sdl.Rect{X: (winWidth - t.w) / 2, Y: winHeight / 3, W: t.w, H: t.h}
		if err := e.renderer.Copy(t.texture, nil, dst); err != nil {
			return fmt.Errorf("could not draw state text: %v", err)
		}
	}

	e.renderer.Present()
	return nil
}

func run() error {
	runtime.LockOSThread()

	e := NewEngine()
	defer e.Destroy()

	if err := e.Init(); err != nil {
		return err
	}
	if err := e.Load(); err != nil {
		return err
	}

	for e.Running() {
		e.handleEvents()
		e.update()
		if err := e.paint(); err != nil {
			return err
		}
		sdl.Delay(frameDelayMS)
	}
	return nil
}

func main() {
	if err := run(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
