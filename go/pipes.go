package main

import (
	"fmt"
	"math/rand/v2"
	"path/filepath"

	"github.com/veandco/go-sdl2/img"
	"github.com/veandco/go-sdl2/sdl"
)

// Pipe layout and movement, in window pixels and seconds.
const (
	pipeW       = 78
	pipeGap     = 200 // vertical opening between the two halves of a pair
	pipeSpacing = 230 // horizontal distance between consecutive pairs
	pipeSpeed   = 165 // px/s, leftward
	gapMargin   = 90  // minimum distance from a gap edge to ceiling or ground

	// Horizontal lead-in before the first pair appears.
	firstPipeDelay = 80
)

type Pipes struct {
	pipes   []*pipe
	spawnX  float64
	texture *sdl.Texture
}

// pipe is one top-and-bottom pair.
type pipe struct {
	x    float64
	gapY float64 // vertical center of the opening
}

func NewPipes(r *sdl.Renderer) (*Pipes, error) {
	texture, err := img.LoadTexture(r, filepath.Join(assetDir, "imgs", "pipe.png"))
	if err != nil {
		return nil, fmt.Errorf("could not load pipe sprite: %v", err)
	}

	return &Pipes{texture: texture, spawnX: winWidth + firstPipeDelay}, nil
}

func newPipe() *pipe {
	span := float64(groundY - 2*gapMargin - pipeGap)
	return &pipe{
		x:    winWidth + pipeW,
		gapY: gapMargin + pipeGap/2 + rand.Float64()*span,
	}
}

// Update scrolls the pipes, spawns a new pair on schedule and drops
// pairs that have left the screen.
func (ps *Pipes) Update(dt float64) {
	for _, p := range ps.pipes {
		p.x -= pipeSpeed * dt
	}

	ps.spawnX -= pipeSpeed * dt
	if ps.spawnX <= winWidth {
		ps.pipes = append(ps.pipes, newPipe())
		ps.spawnX = winWidth + pipeSpacing
	}

	kept := ps.pipes[:0]
	for _, p := range ps.pipes {
		if p.x+pipeW > 0 {
			kept = append(kept, p)
		}
	}
	ps.pipes = kept
}

// Hits reports whether the AABB overlaps any pipe.
func (ps *Pipes) Hits(x, y, w, h float64) bool {
	for _, p := range ps.pipes {
		if x+w > p.x && x < p.x+pipeW {
			if y < p.topH() || y+h > p.bottomY() {
				return true
			}
		}
	}
	return false
}

// Reset removes all pipes and restores the initial spawn delay.
func (ps *Pipes) Reset() {
	ps.pipes = nil
	ps.spawnX = winWidth + firstPipeDelay
}

func (ps *Pipes) Paint(r *sdl.Renderer) error {
	for _, p := range ps.pipes {
		if err := p.paint(r, ps.texture); err != nil {
			return err
		}
	}
	return nil
}

func (ps *Pipes) Destroy() {
	if err := ps.texture.Destroy(); err != nil {
		sdl.LogError(sdl.LOG_CATEGORY_APPLICATION, "destroy pipe texture: %s", err)
	}
}

// topH is the height of the top pipe (the gap's upper edge).
func (p *pipe) topH() float64 {
	return p.gapY - pipeGap/2
}

// bottomY is where the bottom pipe starts (the gap's lower edge).
func (p *pipe) bottomY() float64 {
	return p.gapY + pipeGap/2
}

func (p *pipe) paint(r *sdl.Renderer, t *sdl.Texture) error {
	x := int32(p.x)

	// The source sprite has its cap at the top, so the top pipe is
	// drawn flipped to put the cap at the gap.
	top := &sdl.Rect{X: x, Y: 0, W: pipeW, H: int32(p.topH())}
	if err := r.CopyEx(t, nil, top, 0, nil, sdl.FLIP_VERTICAL); err != nil {
		return fmt.Errorf("could not copy top pipe: %v", err)
	}

	bottom := &sdl.Rect{X: x, Y: int32(p.bottomY()), W: pipeW, H: groundY - int32(p.bottomY())}
	if err := r.Copy(t, nil, bottom); err != nil {
		return fmt.Errorf("could not copy bottom pipe: %v", err)
	}
	return nil
}
