//! Boot: wires up the canvas, input handlers, asset loading, and the
//! fixed-timestep render loop.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, KeyboardEvent, MouseEvent};

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

    let ctx: CanvasRenderingContext2d =
        canvas.get_context("2d")?.ok_or("no 2d context")?.dyn_into()?;

    let game = Rc::new(RefCell::new(Game::new()));

    install_input(&document, &canvas, &game);

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
}

/// Creates an `HtmlImageElement`, fires `on_done` once it has loaded.
fn load_image(src: &str, on_done: impl Fn() + 'static) -> HtmlImageElement {
    let img = HtmlImageElement::new().unwrap();
    let cb = Closure::<dyn FnMut()>::new(move || on_done());
    img.set_onload(Some(cb.as_ref().unchecked_ref()));
    cb.forget();
    img.set_src(src);
    img
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    let _ = web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref());
}
