//! Game state and simulation: the state machine, physics, pipes, scoring and
//! collision. No rendering or DOM access lives here.

use crate::config::*;
use crate::storage::{load_best, save_best};

/// Top-level game state; both input and stepping branch on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Bobbing on the title screen, waiting for the first flap.
    Ready,
    Playing,
    Dead,
}

/// One pipe pair: a left edge and the vertical centre of its gap.
#[derive(Clone, Debug)]
pub struct Pipe {
    pub x: f64,
    pub gap_y: f64,
    /// Set once the bird has passed, so a pipe scores only once.
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

/// The whole simulation: bird, pipes, score, and timers. No DOM access.
#[derive(Debug)]
pub struct Game {
    pub state: State,
    pub bird_y: f64,
    pub bird_v: f64,
    /// Bird rotation in radians; follows velocity.
    pub rot: f64,
    /// Wing-flap animation clock.
    pub anim: f64,
    pub pipes: Vec<Pipe>,
    /// Scrolls left with the pipes; a new pipe spawns when it crosses `W`.
    pub spawn_x: f64,
    pub ground_off: f64,
    pub score: u32,
    pub best: u32,
    pub time: f64,
    /// Seconds since death; drives the impact flash and the restart lockout.
    pub death_flash: f64,
    rng: u32,
}

impl Game {
    /// A fresh game in [`State::Ready`], with the persisted best score loaded.
    pub fn new() -> Self {
        Self {
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
        *self = Self::new();
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

                for p in &mut self.pipes {
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
                let rest_y = GROUND_Y - BIRD_H / 2.0;
                if self.bird_y < rest_y {
                    self.bird_v = (self.bird_v + GRAVITY * dt).min(MAX_FALL);
                    self.bird_y += self.bird_v * dt;
                    self.rot = (self.rot + 4.0 * dt).min(1.6);
                } else {
                    self.bird_y = rest_y;
                    self.rot = 1.6;
                }
            }
        }
    }

    fn check_score(&mut self) {
        for p in &mut self.pipes {
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

        // The ceiling is a wall, not a hazard; same as the original game.
        if by + bh >= GROUND_Y {
            return true;
        }
        // A pipe kills only if the bird overlaps it horizontally and sits
        // outside the gap vertically.
        self.pipes.iter().any(|p| {
            bx + bw > p.x && bx < p.x + PIPE_W && (by < p.top_h() || by + bh > p.bottom_y())
        })
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

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // `super::*` also re-exports the config constants glob-imported above.
    use super::*;

    #[test]
    fn pipe_gap_edges() {
        let p = Pipe {
            x: 0.0,
            gap_y: 400.0,
            scored: false,
        };
        assert_eq!(p.top_h(), 400.0 - PIPE_GAP / 2.0);
        assert_eq!(p.bottom_y(), 400.0 + PIPE_GAP / 2.0);
        // The gap between the edges is exactly PIPE_GAP.
        assert_eq!(p.bottom_y() - p.top_h(), PIPE_GAP);
    }

    #[test]
    fn rng_is_deterministic_and_in_unit_range() {
        let mut a = Game::new();
        let mut b = Game::new();
        for _ in 0..1000 {
            let x = a.rand();
            assert!((0.0..1.0).contains(&x), "rand out of range: {x}");
            assert_eq!(x, b.rand(), "same seed must give same sequence");
        }
    }

    #[test]
    fn flap_from_ready_starts_playing() {
        let mut g = Game::new();
        assert_eq!(g.state, State::Ready);
        g.flap();
        assert_eq!(g.state, State::Playing);
        assert_eq!(g.bird_v, FLAP_V);
    }

    #[test]
    fn flap_while_playing_applies_impulse() {
        let mut g = Game::new();
        g.state = State::Playing;
        g.bird_v = 300.0; // falling
        g.flap();
        assert_eq!(g.bird_v, FLAP_V);
        assert_eq!(g.state, State::Playing);
    }

    #[test]
    fn dead_tap_is_locked_out_then_resets() {
        let mut g = Game::new();
        g.state = State::Dead;
        g.score = 4;
        g.death_flash = 0.0; // too soon
        g.flap();
        assert_eq!(g.state, State::Dead, "tap during lockout must not restart");

        g.death_flash = 0.7; // past the lockout
        g.flap();
        assert_eq!(g.state, State::Ready, "tap after lockout restarts");
        assert_eq!(g.score, 0);
    }

    #[test]
    fn scoring_increments_once_per_pipe() {
        let mut g = Game::new();
        // A pipe fully behind the bird should score exactly once.
        g.pipes.push(Pipe {
            x: BIRD_X - PIPE_W - 1.0,
            gap_y: 400.0,
            scored: false,
        });
        g.check_score();
        assert_eq!(g.score, 1);
        assert!(g.pipes[0].scored);
        g.check_score();
        assert_eq!(g.score, 1, "already-scored pipe must not count again");
    }

    #[test]
    fn pipe_not_yet_passed_does_not_score() {
        let mut g = Game::new();
        g.pipes.push(Pipe {
            x: BIRD_X,
            gap_y: 400.0,
            scored: false,
        });
        g.check_score();
        assert_eq!(g.score, 0);
    }

    #[test]
    fn spawned_pipes_keep_their_gap_inside_the_margins() {
        let mut g = Game::new();
        g.flap(); // Ready -> Playing
        let mut steps = 0;
        while g.pipes.is_empty() && steps < 1000 {
            g.step(FIXED_DT);
            steps += 1;
        }
        let p = g.pipes.first().expect("a pipe should spawn within seconds");
        assert!(p.x >= W, "pipes spawn off-screen to the right");
        // step() places the gap at least `margin` (90) from both the top of
        // the screen and the ground.
        assert!(p.top_h() >= 90.0, "top pipe too short: {}", p.top_h());
        assert!(
            p.bottom_y() <= GROUND_Y - 90.0,
            "gap too low: {}",
            p.bottom_y()
        );
    }

    #[test]
    fn ground_is_lethal_open_sky_is_not() {
        let mut g = Game::new();
        g.bird_y = H * 0.42; // mid-air, no pipes
        assert!(!g.collides());

        g.bird_y = GROUND_Y + BIRD_H; // well into the ground
        assert!(g.collides());
    }

    #[test]
    fn flying_through_the_gap_is_safe_hitting_a_pipe_is_not() {
        let mut g = Game::new();
        g.bird_y = 400.0;
        // Pipe overlapping the bird horizontally, gap centered on the bird.
        g.pipes.push(Pipe {
            x: BIRD_X - PIPE_W / 2.0,
            gap_y: 400.0,
            scored: false,
        });
        assert!(!g.collides(), "centered in the gap should be clear");

        // Move the gap up so the bird is now inside the bottom pipe.
        g.pipes[0].gap_y = 100.0;
        assert!(g.collides(), "outside the gap should collide");
    }

    #[test]
    fn ceiling_clamps_rather_than_kills() {
        let mut g = Game::new();
        g.state = State::Playing;
        g.bird_y = -100.0; // above the top of the screen
        g.bird_v = -200.0;
        g.step(FIXED_DT);
        let ceiling = BIRD_H / 2.0 - HIT_INSET_Y;
        assert!(g.bird_y >= ceiling, "bird should be clamped at the ceiling");
        assert_eq!(
            g.state,
            State::Playing,
            "the ceiling is a wall, not a hazard"
        );
    }

    #[test]
    fn reset_preserves_best_and_clears_the_run() {
        let mut g = Game::new();
        g.best = 9;
        g.score = 3;
        g.state = State::Dead;
        g.death_flash = 1.0;
        g.reset();
        assert_eq!(g.state, State::Ready);
        assert_eq!(g.best, 9, "best score survives a reset");
        assert_eq!(g.score, 0);
    }

    #[test]
    fn dying_records_a_new_best() {
        let mut g = Game::new();
        g.score = 7;
        g.die();
        assert_eq!(g.state, State::Dead);
        assert_eq!(g.best, 7);
    }
}
