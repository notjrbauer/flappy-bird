//! Tunable constants and sprite-atlas coordinates.
//!
//! Everything is expressed in the logical playfield units below; the canvas
//! transform scales them to the device.

/// Logical playfield width.
pub const W: f64 = 480.0;
/// Logical playfield height.
pub const H: f64 = 800.0;

// --- Atlas source rects, measured off assets/FlappySprite.png (512x512) ------

/// Top-left of each 17x12 bird frame; they share a row.
pub const BIRD_FRAMES: [(f64, f64); 3] = [(3.0, 491.0), (31.0, 491.0), (59.0, 491.0)];
pub const BIRD_SW: f64 = 17.0;
pub const BIRD_SH: f64 = 12.0;

pub const SKY_SRC: (f64, f64, f64, f64) = (146.0, 0.0, 145.0, 187.0);
/// The striped base strip; it tiles horizontally so the scroll is visible.
pub const GROUND_SRC: (f64, f64, f64, f64) = (292.0, 0.0, 168.0, 56.0);
pub const GROUND_TILE_W: f64 = 336.0;

// pipe.png is 52x320: a 24px cap at full width, then a body inset 2px per side.
pub const PIPE_CAP_SH: f64 = 24.0;
pub const PIPE_BODY_SY: f64 = 24.0;
pub const PIPE_BODY_SX: f64 = 2.0;
pub const PIPE_BODY_SW: f64 = 48.0;
pub const PIPE_BODY_SH: f64 = 296.0;

// --- Drawn sizes -------------------------------------------------------------

pub const BIRD_W: f64 = 51.0;
pub const BIRD_H: f64 = 36.0;
pub const PIPE_W: f64 = 78.0;
pub const PIPE_CAP_H: f64 = 36.0;
pub const PIPE_INSET: f64 = 3.0;
pub const GROUND_H: f64 = 112.0;
pub const GROUND_Y: f64 = H - GROUND_H;

// --- Tuning ------------------------------------------------------------------

pub const GRAVITY: f64 = 1500.0;
pub const FLAP_V: f64 = -450.0;
pub const MAX_FALL: f64 = 700.0;
pub const PIPE_SPEED: f64 = 165.0;
pub const PIPE_GAP: f64 = 200.0;
pub const PIPE_SPACING: f64 = 230.0;
pub const BIRD_X: f64 = 120.0;

// Bird hitbox is inset from the drawn sprite so near-misses feel fair.
pub const HIT_INSET_X: f64 = 8.0;
pub const HIT_INSET_Y: f64 = 6.0;

/// Physics timestep. The world advances in fixed slices of this length.
pub const FIXED_DT: f64 = 1.0 / 120.0;
