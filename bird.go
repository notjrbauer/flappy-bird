package main

import (
	"path/filepath"
	"sync"

	"github.com/veandco/go-sdl2/sdl"
	img "github.com/veandco/go-sdl2/sdl_image"
)

const jumpSpeed = 5

type Bird struct {
	mu      sync.RWMutex
	time    int
	speed   int32
	x, y    int32
	w, h    int32
	texture *sdl.Texture
	rects   []*sdl.Rect
}

func NewBird(r *sdl.Renderer) (*Bird, error) {
	var rects []*sdl.Rect
	assetDir := filepath.Base("../assets/")

	texture, err := img.LoadTexture(r, filepath.Join(assetDir, "imgs", "FlappySprite.png"))
	if err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "Loading sprite: %s\n", err)
	}

	for i := 0; i < 3; i++ {
		rect := &sdl.Rect{int32(28 * i), 490, 20, 20}
		rects = append(rects, rect)
	}

	sy := int32(WinHeight / 2)
	b := &Bird{x: 0, y: sy, w: 60, h: 43, speed: 1, rects: rects, texture: texture}

	return b, nil
}

func (b *Bird) Paint(r *sdl.Renderer) error {
	b.mu.RLock()
	defer b.mu.RUnlock()

	i := b.time / 2 % len(b.rects)

	rect := &sdl.Rect{X: b.x, Y: b.y, W: b.w, H: b.h}

	err := r.Copy(b.texture, b.rects[i], rect)
	if err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "Paint: %s\n", err)
		return err
	}

	return nil
}

func (b *Bird) Update() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.time++
}

func (b *Bird) Jump(r *sdl.Renderer) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.speed = -jumpSpeed
}

func (b *Bird) Destroy() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.texture.Destroy()
}
