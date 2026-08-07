//! Game state and simulation: the state machine, physics, pipes, scoring and
//! collision. No rendering or DOM access lives here.

use crate::config::*;
use crate::storage::{load_best, save_best};

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Ready,
    Playing,
    Dead,
}

pub struct Pipe {
    pub x: f64,
    pub gap_y: f64,
    pub scored: bool,
}

impl Pipe {
    /// Height of the top pipe (the gap's upper edge).
    pub fn top_h(&self) -> f64 {
        self.gap_y - PIPE_GAP / 2.0
    }
    /// Y of the bottom pipe's cap (the gap's lower edge).
    pub fn bottom_y(&self) -> f64 {
        self.gap_y + PIPE_GAP / 2.0
    }
}

pub struct Game {
    pub state: State,
    pub bird_y: f64,
    pub bird_v: f64,
    pub rot: f64,
    pub anim: f64,
    pub pipes: Vec<Pipe>,
    pub spawn_x: f64,
    pub ground_off: f64,
    pub score: u32,
    pub best: u32,
    pub time: f64,
    pub death_flash: f64,
    rng: u32,
}

impl Game {
    pub fn new() -> Self {
        Game {
            state: State::Ready,
            bird_y: H * 0.42,
            bird_v: 0.0,
            rot: 0.0,
            anim: 0.0,
            pipes: Vec::new(),
            spawn_x: W + 80.0,
            ground_off: 0.0,
            score: 0,
            best: load_best(),
            time: 0.0,
            death_flash: 0.0,
            rng: 0x2545_F491,
        }
    }

    // xorshift; a full rand crate isn't worth the bytes here.
    fn rand(&mut self) -> f64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x >> 8) as f64 / 16_777_216.0
    }

    fn reset(&mut self) {
        let best = self.best;
        let rng = self.rng;
        *self = Game::new();
        self.best = best;
        self.rng = rng;
    }

    /// A tap / click / flap key. Behavior depends on the current state.
    pub fn flap(&mut self) {
        match self.state {
            State::Ready => {
                self.state = State::Playing;
                self.bird_v = FLAP_V;
            }
            State::Playing => self.bird_v = FLAP_V,
            State::Dead => {
                // Small delay so the death tap doesn't instantly restart.
                if self.death_flash > 0.6 {
                    self.reset();
                }
            }
        }
    }

    /// Advances the world by one fixed timestep.
    pub fn step(&mut self, dt: f64) {
        self.time += dt;
        self.anim += dt;

        match self.state {
            State::Ready => {
                self.bird_y = H * 0.42 + (self.time * 4.0).sin() * 10.0;
                self.rot = 0.0;
                self.ground_off += PIPE_SPEED * dt;
            }
            State::Playing => {
                self.bird_v = (self.bird_v + GRAVITY * dt).min(MAX_FALL);
                self.bird_y += self.bird_v * dt;
                self.ground_off += PIPE_SPEED * dt;

                let ceiling = BIRD_H / 2.0 - HIT_INSET_Y;
                if self.bird_y < ceiling {
                    self.bird_y = ceiling;
                    self.bird_v = self.bird_v.max(0.0);
                }

                // Nose up sharply on a flap, tip down as the fall accelerates.
                let target = if self.bird_v < 0.0 {
                    -0.45
                } else {
                    (self.bird_v / MAX_FALL) * 1.4
                };
                self.rot += (target - self.rot) * (1.0 - (-10.0 * dt).exp());

                for p in self.pipes.iter_mut() {
                    p.x -= PIPE_SPEED * dt;
                }
                self.spawn_x -= PIPE_SPEED * dt;
                if self.spawn_x <= W {
                    let margin = 90.0;
                    let span = GROUND_Y - margin * 2.0 - PIPE_GAP;
                    let gap_y = margin + PIPE_GAP / 2.0 + self.rand() * span;
                    self.pipes.push(Pipe {
                        x: W + PIPE_W,
                        gap_y,
                        scored: false,
                    });
                    self.spawn_x = W + PIPE_SPACING;
                }
                self.pipes.retain(|p| p.x + PIPE_W > -10.0);

                self.check_score();
                if self.collides() {
                    self.die();
                }
            }
            State::Dead => {
                self.death_flash += dt;
                if self.bird_y < GROUND_Y - BIRD_H / 2.0 {
                    self.bird_v = (self.bird_v + GRAVITY * dt).min(MAX_FALL);
                    self.bird_y += self.bird_v * dt;
                    self.rot = (self.rot + 4.0 * dt).min(1.6);
                } else {
                    self.bird_y = GROUND_Y - BIRD_H / 2.0;
                    self.rot = 1.6;
                }
            }
        }
    }

    fn check_score(&mut self) {
        for p in self.pipes.iter_mut() {
            if !p.scored && p.x + PIPE_W < BIRD_X {
                p.scored = true;
                self.score += 1;
            }
        }
    }

    fn collides(&self) -> bool {
        let bx = BIRD_X - BIRD_W / 2.0 + HIT_INSET_X;
        let by = self.bird_y - BIRD_H / 2.0 + HIT_INSET_Y;
        let bw = BIRD_W - HIT_INSET_X * 2.0;
        let bh = BIRD_H - HIT_INSET_Y * 2.0;

        // The ceiling is a wall, not a hazard — same as the original game.
        if by + bh >= GROUND_Y {
            return true;
        }
        for p in &self.pipes {
            if bx + bw > p.x && bx < p.x + PIPE_W {
                if by < p.top_h() || by + bh > p.bottom_y() {
                    return true;
                }
            }
        }
        false
    }

    fn die(&mut self) {
        self.state = State::Dead;
        self.death_flash = 0.0;
        self.bird_v = self.bird_v.min(0.0);
        if self.score > self.best {
            self.best = self.score;
            save_best(self.best);
        }
    }
}
