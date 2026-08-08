//! High-score persistence via `localStorage`.
//!
//! The real implementation talks to the browser through `web-sys` and is only
//! built for `wasm32`. On other targets (host unit tests) it degrades to a
//! no-op so the game logic stays testable without a DOM.

const BEST_KEY: &str = "flappy_best";

/// Reads the stored best score, or 0 if none / unavailable.
#[cfg(target_arch = "wasm32")]
pub fn load_best() -> u32 {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(BEST_KEY).ok().flatten())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Persists the best score. Silently no-ops if storage is unavailable.
#[cfg(target_arch = "wasm32")]
pub fn save_best(v: u32) {
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.set_item(BEST_KEY, &v.to_string());
    }
}

/// Host stand-in: there is no storage, so the best score starts at 0.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_best() -> u32 {
    0
}

/// Host stand-in: the score is dropped.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_best(_v: u32) {}
