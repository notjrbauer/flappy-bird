package main

import "C"

import (
	"fmt"
	"path/filepath"
	"runtime"

	"github.com/veandco/go-sdl2/sdl"
	"github.com/veandco/go-sdl2/sdl_ttf"
)

const (
	// Window title
	WinTitle = "Go SDL2 Example"
	// Window width
	WinWidth = 480
	// Window height
	WinHeight = 800
)

// Game states
const (
	StateRun = iota
	StateFlap
	StateDead
)

var birdy int32

// States text
var stateText = map[int]string{
	StateRun:  "RUN",
	StateFlap: "FLAP",
	StateDead: "DEAD",
}

// SDL engine structure
type Engine struct {
	State     int
	Window    *sdl.Window
	Renderer  *sdl.Renderer
	Scene     *Scene
	Entity    *Bird
	Font      *ttf.Font
	StateText map[int]*Text
	running   bool
}

// State text structure
type Text struct {
	Width   int32
	Height  int32
	Texture *sdl.Texture
}

// Returns new engine
func NewEngine() (e *Engine) {
	e = &Engine{}
	e.running = true
	return
}

// Initializes SDL
func (e *Engine) Init() error {
	err := sdl.Init(sdl.INIT_EVERYTHING)
	if err != nil {
		return fmt.Errorf("could not initalize SDL: %v", err)
	}

	if err := ttf.Init(); err != nil {
		return fmt.Errorf("could not initalize TTF: %v", err)
	}

	e.Window, err = sdl.CreateWindow(WinTitle, sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED, WinWidth, WinHeight, sdl.WINDOW_SHOWN)
	if err != nil {
		return fmt.Errorf("could no create window: %v", err)
	}

	e.Renderer, err = sdl.CreateRenderer(e.Window, -1, sdl.RENDERER_ACCELERATED)

	if err != nil {
		return fmt.Errorf("could no create renderer: %v", err)
	}

	e.Scene, err = NewScene(e.Renderer)

	return err
}

// Destroys SDL and releases the memory
func (e *Engine) Destroy() {
	for _, v := range e.StateText {
		v.Texture.Destroy()
	}

	e.Font.Close()
	e.Renderer.Destroy()
	e.Window.Destroy()
	e.Scene.Destroy()

	ttf.Quit()
	sdl.Quit()
}

// Quits main loop
func (e *Engine) Quit() {
	e.running = false
}

// Checks if loop is running
func (e *Engine) Running() bool {
	return e.running
}

// Loads sprite
//func (e *Engine) LoadSprite(file string) error {
//  texture, err := img.LoadTexture(e.Renderer, file)
//  e.Sprite = append(e.Sprite, texture)
//  return err
//}

//func (e *Engine) LoadBackground(file string) error {
//  return err
//}

// Loads ttf font
func (e *Engine) LoadFont(file string, size int) (err error) {
	e.Font, err = ttf.OpenFont(file, size)
	return
}

// Loads resources
func (e *Engine) Load() {
	assetDir := filepath.Base("../assets/")

	err := e.LoadFont(filepath.Join(assetDir, "fonts", "Flappy.ttf"), 24)
	if err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "LoadTexture: %s\n", err)
	}

	e.StateText = map[int]*Text{}
	for k, v := range stateText {
		t, _ := e.RenderText(v, sdl.Color{0, 0, 0, 0})
		_, _, tW, tH, _ := t.Query()
		e.StateText[k] = &Text{tW, tH, t}
	}
}

// Renders texture from ttf font
func (e *Engine) RenderText(text string, color sdl.Color) (texture *sdl.Texture, err error) {
	surface, err := e.Font.RenderUTF8_Blended(text, color)
	if err != nil {
		return
	}
	defer surface.Free()

	texture, err = e.Renderer.CreateTextureFromSurface(surface)
	return
}

func run() {
	runtime.LockOSThread()
	e := NewEngine()

	// Initialize SDL
	err := e.Init()
	if err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "Init: %s\n", err)
	}
	defer e.Destroy()

	// Sprite size
	const n = 128

	// Sprite rects
	var rects []*sdl.Rect
	for x := 0; x < 4; x++ {
		rect := &sdl.Rect{int32(n * x), 0, n, n}
		rects = append(rects, rect)
	}

	// Load resources
	e.Load()

	// Play music

	var frame int = 0
	var alpha uint8 = 255
	var showText bool = true

	var text *Text = e.StateText[StateRun]

	for e.Running() {

		for event := sdl.PollEvent(); event != nil; event = sdl.PollEvent() {
			switch t := event.(type) {
			case *sdl.QuitEvent:
				e.Quit()

			case *sdl.MouseButtonEvent:
				if t.Type == sdl.MOUSEBUTTONDOWN && t.Button == sdl.BUTTON_LEFT {
					alpha = 255
					showText = true

					if e.State == StateRun {
						text = e.StateText[StateFlap]
						e.State = StateFlap
					} else if e.State == StateFlap {
						text = e.StateText[StateDead]
						e.State = StateDead
					} else if e.State == StateDead {
						text = e.StateText[StateRun]
						e.State = StateRun
					}
				}

			case *sdl.KeyDownEvent:
				s := sdl.GetKeyboardState()
				if s[sdl.SCANCODE_A] != 0 {
					e.Scene.bird.x -= 2
				}
				if s[sdl.SCANCODE_D] != 0 {
					e.Scene.bird.x += 2
				}

				if s[sdl.SCANCODE_W] != 0 {
					e.Scene.bird.y -= 2
				}
				if s[sdl.SCANCODE_S] != 0 {
					e.Scene.bird.y += 2
				}

				if t.Keysym.Scancode == sdl.SCANCODE_ESCAPE || t.Keysym.Scancode == sdl.SCANCODE_AC_BACK {
					e.Quit()
				}
			}
		}

		e.Scene.Update(e.Renderer)
		e.Renderer.Clear()

		w, h := e.Window.GetSize()
		x, y := int32(w/2), int32(h/2)

		switch e.State {
		case StateRun:

		case StateFlap:

		case StateDead:
		}

		if showText {
			text.Texture.SetAlphaMod(alpha)
			e.Renderer.Copy(text.Texture, nil, &sdl.Rect{x - (text.Width / 2), y - n*1.5, text.Width, text.Height})
		}

		e.Scene.Paint(e.Renderer)
		sdl.Delay(50)

		frame += 1
		if frame/2 >= 2 {
			frame = 0
		}

		alpha -= 10
		if alpha <= 10 {
			alpha = 255
			showText = false
		}
	}
}

// Exports function to C
//export main2
func main2() {
	run()
}

// Go main function
func main() {
	run()
}
