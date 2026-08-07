//! Flappy Bird — Rust + WebAssembly port.
//!
//! Module layout:
//! - [`config`]  tunable constants and sprite-atlas coordinates
//! - [`game`]    state machine, physics, collision, scoring (no DOM)
//! - [`render`]  canvas 2D drawing of a [`game::Game`]
//! - [`storage`] high-score persistence via `localStorage`
//! - [`app`]     boot: canvas, input, asset loading, the render loop
//!
//! `config`, `game`, and `storage` are portable and unit-tested on the host
//! target. `app` and `render` touch the DOM through `web-sys`, so they are
//! compiled only for `wasm32`.

// This crate is a wasm binary: the DOM-facing modules (`app`, `render`) and the
// game API they drive are only reachable on wasm32. On the host target we build
// only to run the unit tests, where that code reads as dead. Silence it there;
// wasm builds stay strict.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

mod config;
mod game;
mod storage;

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod render;

/// Called automatically when the wasm module is instantiated.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    app::run()
}
