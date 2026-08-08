package main

import (
	"fmt"
	"path/filepath"

	"github.com/veandco/go-sdl2/img"
	"github.com/veandco/go-sdl2/sdl"
)

// The background atlas rect below is stretched over the whole window;
// its grass begins at source y=187 of 256, which lands at window y=584.
// That line is the ground: bottom pipes stand on it and touching it
// ends the run.
const groundY = 584

type Scene struct {
	background *sdl.Texture
	bird       *Bird
	pipes      *Pipes
}

func NewScene(r *sdl.Renderer) (*Scene, error) {
	background, err := img.LoadTexture(r, filepath.Join(assetDir, "imgs", "FlappySprite.png"))
	if err != nil {
		return nil, fmt.Errorf("could not load background: %v", err)
	}

	bird, err := NewBird(r)
	if err != nil {
		return nil, err
	}
	pipes, err := NewPipes(r)
	if err != nil {
		return nil, err
	}
	return &Scene{background: background, bird: bird, pipes: pipes}, nil
}

// Paint draws the scene back to front. Present is left to the caller so
// overlay text can be drawn on top.
func (s *Scene) Paint(r *sdl.Renderer) error {
	if err := r.Clear(); err != nil {
		return fmt.Errorf("could not clear renderer: %v", err)
	}
	src := &sdl.Rect{X: 146, Y: 0, W: 145, H: 256}
	if err := r.Copy(s.background, src, nil); err != nil {
		return fmt.Errorf("could not copy background: %v", err)
	}
	if err := s.pipes.Paint(r); err != nil {
		return err
	}
	return s.bird.Paint(r)
}

// Update advances bird physics and pipe movement by one step.
func (s *Scene) Update(dt float64) {
	s.bird.Update(dt)
	s.pipes.Update(dt)
}

// Collides reports whether the bird is touching the ground or a pipe.
func (s *Scene) Collides() bool {
	x, y, w, h := s.bird.hitbox()
	if y+h >= groundY {
		return true
	}
	return s.pipes.Hits(x, y, w, h)
}

// Reset starts a fresh run.
func (s *Scene) Reset() {
	s.bird.Reset()
	s.pipes.Reset()
}

func (s *Scene) Destroy() {
	s.bird.Destroy()
	s.pipes.Destroy()
	if err := s.background.Destroy(); err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "destroy background: %s", err)
	}
}
