package main

import (
	"fmt"
	"path/filepath"

	"github.com/veandco/go-sdl2/img"
	"github.com/veandco/go-sdl2/sdl"
)

// Bird geometry and physics, in window pixels and seconds.
const (
	birdX      = 120 // fixed horizontal position of the sprite's left edge
	birdW      = 60
	birdH      = 43
	birdStartY = winHeight * 42 / 100

	gravity      = 1500 // px/s^2
	flapSpeed    = 450  // px/s, upward
	maxFallSpeed = 700  // px/s

	// The hitbox is inset from the drawn sprite so near-misses feel fair.
	hitInsetX = 8
	hitInsetY = 6
)

type Bird struct {
	y, vy   float64
	frame   int
	texture *sdl.Texture
	rects   []*sdl.Rect
}

func NewBird(r *sdl.Renderer) (*Bird, error) {
	texture, err := img.LoadTexture(r, filepath.Join(assetDir, "imgs", "FlappySprite.png"))
	if err != nil {
		return nil, fmt.Errorf("could not load bird sprite: %v", err)
	}

	// Three wing frames sharing a row of the atlas.
	var rects []*sdl.Rect
	for i := 0; i < 3; i++ {
		rects = append(rects, &sdl.Rect{X: int32(28 * i), Y: 490, W: 20, H: 20})
	}

	return &Bird{y: birdStartY, texture: texture, rects: rects}, nil
}

func (b *Bird) Paint(r *sdl.Renderer) error {
	i := b.frame / 6 % len(b.rects)
	dst := &sdl.Rect{X: birdX, Y: int32(b.y), W: birdW, H: birdH}
	if err := r.Copy(b.texture, b.rects[i], dst); err != nil {
		return fmt.Errorf("could not copy bird: %v", err)
	}
	return nil
}

// Animate advances the wing animation without moving the bird.
func (b *Bird) Animate() {
	b.frame++
}

// Flap gives the bird its upward impulse.
func (b *Bird) Flap() {
	b.vy = -flapSpeed
}

// Update integrates gravity for one step. The ceiling clamps rather
// than kills, matching the original game.
func (b *Bird) Update(dt float64) {
	b.Animate()

	b.vy += gravity * dt
	if b.vy > maxFallSpeed {
		b.vy = maxFallSpeed
	}
	b.y += b.vy * dt

	if b.y < -hitInsetY {
		b.y = -hitInsetY
		if b.vy < 0 {
			b.vy = 0
		}
	}
}

// Fall drops the dead bird until it rests on the ground.
func (b *Bird) Fall(dt float64) {
	b.vy += gravity * dt
	if b.vy > maxFallSpeed {
		b.vy = maxFallSpeed
	}
	b.y += b.vy * dt

	if b.y > groundY-birdH {
		b.y = groundY - birdH
		b.vy = 0
	}
}

// hitbox returns the bird's collision AABB.
func (b *Bird) hitbox() (x, y, w, h float64) {
	return birdX + hitInsetX, b.y + hitInsetY, birdW - 2*hitInsetX, birdH - 2*hitInsetY
}

// Reset puts the bird back at its starting position.
func (b *Bird) Reset() {
	b.y = birdStartY
	b.vy = 0
	b.frame = 0
}

func (b *Bird) Destroy() {
	if err := b.texture.Destroy(); err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "destroy bird texture: %s", err)
	}
}
