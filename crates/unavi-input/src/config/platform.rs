//! Per-platform constants that have no other sane home: the browser decides
//! them, not UNAVI's own config.

/// Pointer lock reports mouse-motion deltas in different units per browser,
/// so the configured sensitivity means the same thing everywhere.
#[cfg(target_family = "wasm")]
#[must_use]
pub fn mouse_motion_scale() -> f32 {
    use std::sync::OnceLock;
    static IS_FIREFOX: OnceLock<bool> = OnceLock::new();

    let is_firefox = *IS_FIREFOX.get_or_init(|| {
        web_sys::window()
            .and_then(|window| window.navigator().user_agent().ok())
            .is_some_and(|agent| agent.contains("Firefox"))
    });

    if is_firefox { 12.0 } else { 0.7 }
}

#[cfg(not(target_family = "wasm"))]
#[must_use]
pub const fn mouse_motion_scale() -> f32 {
    1.0
}
