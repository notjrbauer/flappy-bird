package main

import (
	"fmt"
	"math/rand"
	"path/filepath"
	"sync"
	"time"

	"github.com/veandco/go-sdl2/sdl"
	img "github.com/veandco/go-sdl2/sdl_image"
)

type Pipes struct {
	mu      sync.RWMutex
	speed   int32
	pipes   []*pipe
	texture *sdl.Texture
}

func NewPipes(r *sdl.Renderer) (*Pipes, error) {
	assetDir := filepath.Base("../assets/")
	t, err := img.LoadTexture(r, filepath.Join(assetDir, "imgs", "pipe.png"))
	if err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "NewPipes: %s\n", err)
		return nil, err
	}

	ps := &Pipes{
		texture: t,
		speed:   2,
	}

	go func() {
		for {
			ps.mu.Lock()
			ps.pipes = append(ps.pipes, NewPipe())
			ps.mu.Unlock()
			time.Sleep(time.Second)
		}
	}()

	return ps, nil
}

func (ps *Pipes) Paint(r *sdl.Renderer) error {
	ps.mu.RLock()
	defer ps.mu.RUnlock()

	for _, p := range ps.pipes {
		if err := p.paint(r, ps.texture); err != nil {
			return err
		}
	}
	return nil
}

func (ps *Pipes) Update() {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	for _, p := range ps.pipes {
		p.mu.Lock()
		p.x -= 5
		p.mu.Unlock()
	}
}
func (ps *Pipes) Destroy() {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	ps.texture.Destroy()
}

type pipe struct {
	mu     sync.RWMutex
	x      int32
	w      int32
	h      int32
	invert bool
}

func NewPipe() *pipe {
	return &pipe{
		x:      600,
		w:      50,
		h:      100 + int32(rand.Intn(300)),
		invert: rand.Float32() > 0.5,
	}
}

func (p *pipe) paint(r *sdl.Renderer, t *sdl.Texture) error {
	p.mu.RLock()
	defer p.mu.RUnlock()
	rect := &sdl.Rect{X: p.x, Y: 800 - p.h, W: p.w, H: p.h}

	flip := sdl.FLIP_NONE
	if p.invert {
		rect.Y = 0
		flip = sdl.FLIP_VERTICAL
	}
	if err := r.CopyEx(t, nil, rect, 0, nil, flip); err != nil {
		return fmt.Errorf("could not copy background: %v", err)
	}
	return nil
}
