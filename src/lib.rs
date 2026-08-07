use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, KeyboardEvent, MouseEvent,
};

// ---------------------------------------------------------------------------
// Logical playfield. Everything below is expressed in these units; the canvas
// transform scales them to the device.
// ---------------------------------------------------------------------------

const W: f64 = 480.0;
const H: f64 = 800.0;

// Atlas source rects, measured off assets/FlappySprite.png (512x512).
const BIRD_FRAMES: [(f64, f64); 3] = [(3.0, 491.0), (31.0, 491.0), (59.0, 491.0)];
const BIRD_SW: f64 = 17.0;
const BIRD_SH: f64 = 12.0;

const SKY_SRC: (f64, f64, f64, f64) = (146.0, 0.0, 145.0, 187.0);
// The striped base strip; it tiles horizontally so the scroll is visible.
const GROUND_SRC: (f64, f64, f64, f64) = (292.0, 0.0, 168.0, 56.0);
const GROUND_TILE_W: f64 = 336.0;

// pipe.png is 52x320: a 24px cap at full width, then a body inset 2px per side.
const PIPE_CAP_SH: f64 = 24.0;
const PIPE_BODY_SY: f64 = 24.0;
const PIPE_BODY_SX: f64 = 2.0;
const PIPE_BODY_SW: f64 = 48.0;
const PIPE_BODY_SH: f64 = 296.0;

// Drawn sizes.
const BIRD_W: f64 = 51.0;
const BIRD_H: f64 = 36.0;
const PIPE_W: f64 = 78.0;
const PIPE_CAP_H: f64 = 36.0;
const PIPE_INSET: f64 = 3.0;
const GROUND_H: f64 = 112.0;
const GROUND_Y: f64 = H - GROUND_H;

// Tuning.
const GRAVITY: f64 = 1500.0;
const FLAP_V: f64 = -450.0;
const MAX_FALL: f64 = 700.0;
const PIPE_SPEED: f64 = 165.0;
const PIPE_GAP: f64 = 200.0;
const PIPE_SPACING: f64 = 230.0;
const BIRD_X: f64 = 120.0;

// Bird hitbox is inset from the drawn sprite so near-misses feel fair.
const HIT_INSET_X: f64 = 8.0;
const HIT_INSET_Y: f64 = 6.0;

const FIXED_DT: f64 = 1.0 / 120.0;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Ready,
    Playing,
    Dead,
}

struct Pipe {
    x: f64,
    gap_y: f64,
    scored: bool,
}

impl Pipe {
    fn top_h(&self) -> f64 {
        self.gap_y - PIPE_GAP / 2.0
    }
    fn bottom_y(&self) -> f64 {
        self.gap_y + PIPE_GAP / 2.0
    }
}

struct Game {
    state: State,
    bird_y: f64,
    bird_v: f64,
    rot: f64,
    anim: f64,
    pipes: Vec<Pipe>,
    spawn_x: f64,
    ground_off: f64,
    score: u32,
    best: u32,
    time: f64,
    death_flash: f64,
    rng: u32,
}

impl Game {
    fn new() -> Self {
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

    fn flap(&mut self) {
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

    fn step(&mut self, dt: f64) {
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
                let target = if self.bird_v < 0.0 { -0.45 } else { (self.bird_v / MAX_FALL) * 1.4 };
                self.rot += (target - self.rot) * (1.0 - (-10.0 * dt).exp());

                for p in self.pipes.iter_mut() {
                    p.x -= PIPE_SPEED * dt;
                }
                self.spawn_x -= PIPE_SPEED * dt;
                if self.spawn_x <= W {
                    let margin = 90.0;
                    let span = GROUND_Y - margin * 2.0 - PIPE_GAP;
                    let gap_y = margin + PIPE_GAP / 2.0 + self.rand() * span;
                    self.pipes.push(Pipe { x: W + PIPE_W, gap_y, scored: false });
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

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

struct Assets {
    atlas: HtmlImageElement,
    pipe: HtmlImageElement,
}

fn blit(
    ctx: &CanvasRenderingContext2d,
    img: &HtmlImageElement,
    src: (f64, f64, f64, f64),
    dx: f64,
    dy: f64,
    dw: f64,
    dh: f64,
) {
    let _ = ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        img, src.0, src.1, src.2, src.3, dx, dy, dw, dh,
    );
}

/// Draws a pipe with its cap at `y`, body running down to `y + h`.
/// The cap keeps a fixed height instead of being stretched with the body,
/// which is what the original SDL version did.
fn draw_pipe_upright(ctx: &CanvasRenderingContext2d, img: &HtmlImageElement, x: f64, y: f64, h: f64) {
    let body_h = (h - PIPE_CAP_H).max(0.0);
    if body_h > 0.0 {
        blit(
            ctx,
            img,
            (PIPE_BODY_SX, PIPE_BODY_SY, PIPE_BODY_SW, PIPE_BODY_SH),
            x + PIPE_INSET,
            y + PIPE_CAP_H,
            PIPE_W - PIPE_INSET * 2.0,
            body_h,
        );
    }
    blit(ctx, img, (0.0, 0.0, 52.0, PIPE_CAP_SH), x, y, PIPE_W, PIPE_CAP_H);
}

fn draw(ctx: &CanvasRenderingContext2d, g: &Game, a: &Assets, dpr: f64) {
    let _ = ctx.reset_transform();
    ctx.set_image_smoothing_enabled(false);
    let _ = ctx.scale(dpr, dpr);
    ctx.clear_rect(0.0, 0.0, W, H);

    // Sky, stretched to fill everything above the ground.
    blit(ctx, &a.atlas, SKY_SRC, 0.0, 0.0, W, GROUND_Y);

    // Pipes.
    for p in &g.pipes {
        // Top pipe: same sprite, mirrored vertically about the gap edge.
        ctx.save();
        let _ = ctx.translate(0.0, p.top_h());
        let _ = ctx.scale(1.0, -1.0);
        draw_pipe_upright(ctx, &a.pipe, p.x, 0.0, p.top_h());
        ctx.restore();

        draw_pipe_upright(ctx, &a.pipe, p.x, p.bottom_y(), GROUND_Y - p.bottom_y());
    }

    // Scrolling ground. Positions are rounded and tiles overlap by a pixel;
    // otherwise fractional offsets leave a hairline seam between them.
    let off = -(g.ground_off % GROUND_TILE_W);
    let tiles = (W / GROUND_TILE_W).ceil() as i32 + 1;
    for i in 0..tiles {
        let x = (off + i as f64 * GROUND_TILE_W).round();
        blit(ctx, &a.atlas, GROUND_SRC, x, GROUND_Y, GROUND_TILE_W + 1.0, GROUND_H);
    }

    // Bird, rotated about its centre.
    let frame = if g.state == State::Dead {
        1
    } else {
        ((g.anim * 12.0) as usize) % BIRD_FRAMES.len()
    };
    let (sx, sy) = BIRD_FRAMES[frame];
    ctx.save();
    let _ = ctx.translate(BIRD_X, g.bird_y);
    let _ = ctx.rotate(g.rot);
    blit(
        ctx,
        &a.atlas,
        (sx, sy, BIRD_SW, BIRD_SH),
        -BIRD_W / 2.0,
        -BIRD_H / 2.0,
        BIRD_W,
        BIRD_H,
    );
    ctx.restore();

    draw_hud(ctx, g);
}

fn outlined_text(ctx: &CanvasRenderingContext2d, text: &str, x: f64, y: f64, size: f64) {
    ctx.set_font(&format!("bold {}px 'Courier New', monospace", size));
    ctx.set_text_align("center");
    ctx.set_line_width(size / 6.0);
    ctx.set_line_join("round");
    ctx.set_stroke_style_str("#2c2c2c");
    let _ = ctx.stroke_text(text, x, y);
    ctx.set_fill_style_str("#ffffff");
    let _ = ctx.fill_text(text, x, y);
}

fn draw_hud(ctx: &CanvasRenderingContext2d, g: &Game) {
    match g.state {
        State::Ready => {
            outlined_text(ctx, "TAP OR SPACE", W / 2.0, 200.0, 34.0);
            outlined_text(ctx, "TO FLAP", W / 2.0, 244.0, 34.0);
        }
        State::Playing => {
            outlined_text(ctx, &g.score.to_string(), W / 2.0, 110.0, 64.0);
        }
        State::Dead => {
            // Quick white flash on impact.
            if g.death_flash < 0.15 {
                ctx.set_fill_style_str(&format!("rgba(255,255,255,{})", 1.0 - g.death_flash / 0.15));
                ctx.fill_rect(0.0, 0.0, W, H);
            }
            outlined_text(ctx, "GAME OVER", W / 2.0, 260.0, 52.0);
            outlined_text(ctx, &format!("SCORE {}", g.score), W / 2.0, 330.0, 34.0);
            outlined_text(ctx, &format!("BEST {}", g.best), W / 2.0, 372.0, 34.0);
            if g.death_flash > 0.6 {
                outlined_text(ctx, "TAP TO RETRY", W / 2.0, 440.0, 28.0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

fn load_best() -> u32 {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("flappy_best").ok().flatten())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn save_best(v: u32) {
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.set_item("flappy_best", &v.to_string());
    }
}

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------

fn load_image(src: &str, on_done: impl Fn() + 'static) -> HtmlImageElement {
    let img = HtmlImageElement::new().unwrap();
    let cb = Closure::<dyn FnMut()>::new(move || on_done());
    img.set_onload(Some(cb.as_ref().unchecked_ref()));
    cb.forget();
    img.set_src(src);
    img
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("game")
        .ok_or("missing #game canvas")?
        .dyn_into()?;

    let dpr = window.device_pixel_ratio().max(1.0);
    canvas.set_width((W * dpr) as u32);
    canvas.set_height((H * dpr) as u32);

    let ctx: CanvasRenderingContext2d = canvas.get_context("2d")?.ok_or("no 2d context")?.dyn_into()?;

    let game = Rc::new(RefCell::new(Game::new()));

    // Input.
    {
        let g = game.clone();
        let cb = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            e.prevent_default();
            g.borrow_mut().flap();
        });
        canvas.set_onmousedown(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
    }
    {
        let g = game.clone();
        let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
            e.prevent_default();
            g.borrow_mut().flap();
        });
        let _ = canvas.add_event_listener_with_callback("touchstart", cb.as_ref().unchecked_ref());
        cb.forget();
    }
    {
        let g = game.clone();
        let cb = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            let k = e.key();
            if k == " " || k == "ArrowUp" || k == "w" || k == "W" {
                e.prevent_default();
                g.borrow_mut().flap();
            }
        });
        let _ = document.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // Wait for both textures before starting the loop.
    let pending = Rc::new(RefCell::new(2u32));
    let ready = Rc::new(RefCell::new(false));

    let atlas = {
        let p = pending.clone();
        let r = ready.clone();
        load_image("assets/FlappySprite.png", move || {
            *p.borrow_mut() -= 1;
            if *p.borrow() == 0 {
                *r.borrow_mut() = true;
            }
        })
    };
    let pipe = {
        let p = pending.clone();
        let r = ready.clone();
        load_image("assets/pipe.png", move || {
            *p.borrow_mut() -= 1;
            if *p.borrow() == 0 {
                *r.borrow_mut() = true;
            }
        })
    };
    let assets = Assets { atlas, pipe };

    // Fixed-timestep loop driven by requestAnimationFrame.
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
    let g2 = f.clone();
    let mut last = 0.0f64;
    let mut acc = 0.0f64;

    *g2.borrow_mut() = Some(Closure::new(move |now: f64| {
        if *ready.borrow() {
            if last == 0.0 {
                last = now;
            }
            // Clamp so a backgrounded tab doesn't fast-forward the world.
            let dt = ((now - last) / 1000.0).min(0.25);
            last = now;
            acc += dt;
            while acc >= FIXED_DT {
                game.borrow_mut().step(FIXED_DT);
                acc -= FIXED_DT;
            }
            draw(&ctx, &game.borrow(), &assets, dpr);
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g2.borrow().as_ref().unwrap());
    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    let _ = web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref());
}
