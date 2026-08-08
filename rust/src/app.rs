//! Boot: wires up the canvas, input handlers, asset loading, and the
//! fixed-timestep render loop.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, KeyboardEvent, MouseEvent,
    TouchEvent,
};

use crate::config::{FIXED_DT, H, W};
use crate::game::Game;
use crate::render::{draw, Assets};

/// Entry point, called from the `#[wasm_bindgen(start)]` shim in lib.rs.
pub fn run() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("game")
        .ok_or("missing #game canvas")?
        .dyn_into()?;

    let dpr = window.device_pixel_ratio().max(1.0);
    canvas.set_width((W * dpr) as u32);
    canvas.set_height((H * dpr) as u32);

    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .dyn_into()?;

    let game = Rc::new(RefCell::new(Game::new()));

    install_input(&document, &canvas, &game);

    // The loop idles until both textures have loaded and this reaches zero.
    let pending = Rc::new(Cell::new(2u32));

    let atlas = {
        let p = pending.clone();
        load_image("assets/FlappySprite.png", move || p.set(p.get() - 1))
    };
    let pipe = {
        let p = pending.clone();
        load_image("assets/pipe.png", move || p.set(p.get() - 1))
    };
    let assets = Assets { atlas, pipe };

    // Fixed-timestep loop driven by requestAnimationFrame. The callback has
    // to reschedule itself, hence the shared slot it is stored in.
    let frame_cb = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
    let next_frame = frame_cb.clone();
    let mut last = 0.0f64;
    let mut acc = 0.0f64;

    *frame_cb.borrow_mut() = Some(Closure::new(move |now: f64| {
        if pending.get() == 0 {
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
        let cb = next_frame.borrow();
        request_animation_frame(
            cb.as_ref()
                .expect("frame callback is installed before the loop starts"),
        );
    }));

    request_animation_frame(
        frame_cb
            .borrow()
            .as_ref()
            .expect("frame callback was just installed"),
    );
    Ok(())
}

/// Mouse, touch, and keyboard: all map to a single `flap()`.
fn install_input(
    document: &web_sys::Document,
    canvas: &HtmlCanvasElement,
    game: &Rc<RefCell<Game>>,
) {
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
        let cb = Closure::<dyn FnMut(TouchEvent)>::new(move |e: TouchEvent| {
            e.prevent_default();
            g.borrow_mut().flap();
        });
        let _ = canvas.add_event_listener_with_callback("touchstart", cb.as_ref().unchecked_ref());
        cb.forget();
    }
    {
        let g = game.clone();
        let cb = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            if matches!(e.key().as_str(), " " | "ArrowUp" | "w" | "W") {
                e.prevent_default();
                g.borrow_mut().flap();
            }
        });
        let _ = document.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
        cb.forget();
    }
}

/// Creates an `HtmlImageElement`, fires `on_done` once it has loaded.
fn load_image(src: &str, on_done: impl FnMut() + 'static) -> HtmlImageElement {
    let img = HtmlImageElement::new().expect("failed to create an <img> element");
    let cb = Closure::<dyn FnMut()>::new(on_done);
    img.set_onload(Some(cb.as_ref().unchecked_ref()));
    cb.forget();
    img.set_src(src);
    img
}

/// Schedules `f` for the browser's next animation frame.
fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    let _ = web_sys::window()
        .expect("no window")
        .request_animation_frame(f.as_ref().unchecked_ref());
}
