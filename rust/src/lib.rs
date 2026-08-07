//! Flappy Bird — Rust + WebAssembly port.
//!
//! Module layout:
//! - [`config`]  tunable constants and sprite-atlas coordinates
//! - [`game`]    state machine, physics, collision, scoring (no DOM)
//! - [`render`]  canvas 2D drawing of a [`game::Game`]
//! - [`storage`] high-score persistence via `localStorage`
//! - [`app`]     boot: canvas, input, asset loading, the render loop

mod app;
mod config;
mod game;
mod render;
mod storage;

use wasm_bindgen::prelude::*;

/// Called automatically when the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    app::run()
}
