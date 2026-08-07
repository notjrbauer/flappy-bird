package main

import (
	"path/filepath"

	"github.com/veandco/go-sdl2/sdl"
	img "github.com/veandco/go-sdl2/sdl_image"
)

type Scene struct {
	background *sdl.Texture
	bird       *Bird
	pipes      *Pipes
}

func NewScene(r *sdl.Renderer) (*Scene, error) {
	assetDir := filepath.Base("../assets/")

	bg, err := img.LoadTexture(r, filepath.Join(assetDir, "imgs", "FlappySprite.png"))
	if err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "Loading New Scene %s\n", err)
		return nil, err
	}

	b, err := NewBird(r)
	ps, err := NewPipes(r)
	return &Scene{background: bg, bird: b, pipes: ps}, nil
}

func (s *Scene) Paint(r *sdl.Renderer) error {
	r.Clear()
	if err := r.Copy(s.background, &sdl.Rect{X: 146, Y: 0, W: 145, H: 256}, nil); err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "Loading Background to Scene %s\n", err)
	}
	if err := s.pipes.Paint(r); err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "Loading Bird to Scene %s\n", err)
	}

	if err := s.bird.Paint(r); err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "Loading Bird to Scene %s\n", err)
	}

	r.Present()
	return nil
}

func (s *Scene) Update(r *sdl.Renderer) {
	s.bird.Update()
	s.pipes.Update()
}

func (s *Scene) Destroy() {
	s.bird.Destroy()
	s.pipes.Destroy()
}
