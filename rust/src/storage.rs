//! High-score persistence via `localStorage`.

const BEST_KEY: &str = "flappy_best";

/// Reads the stored best score, or 0 if none / unavailable.
pub fn load_best() -> u32 {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(BEST_KEY).ok().flatten())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Persists the best score. Silently no-ops if storage is unavailable.
pub fn save_best(v: u32) {
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.set_item(BEST_KEY, &v.to_string());
    }
}
