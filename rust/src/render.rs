//! Canvas 2D rendering. Reads a `Game` and draws it; holds no game state.

use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

use crate::config::*;
use crate::game::{Game, State};

/// The two loaded textures: the sprite atlas and the pipe.
pub struct Assets {
    pub atlas: HtmlImageElement,
    pub pipe: HtmlImageElement,
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

/// Renders the whole frame: sky, pipes, ground, bird, then the HUD overlay.
pub fn draw(ctx: &CanvasRenderingContext2d, g: &Game, a: &Assets, dpr: f64) {
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
