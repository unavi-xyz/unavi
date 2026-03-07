const BASE_SENSITIVITY: f32 = 0.08;

#[cfg(target_family = "wasm")]
pub fn sensitivity() -> f32 {
    use std::sync::OnceLock;
    static IS_FIREFOX: OnceLock<bool> = OnceLock::new();

    let is_firefox = *IS_FIREFOX.get_or_init(|| {
        web_sys::window()
            .and_then(|w| w.navigator().user_agent().ok())
            .is_some_and(|ua| ua.contains("Firefox"))
    });

    if is_firefox {
        // Firefox reports different pointer lock deltas, requiring higher sensitivity.
        BASE_SENSITIVITY * 12.0
    } else {
        // Other browsers need a lower sensitivity than native.
        BASE_SENSITIVITY * 0.7
    }
}

#[cfg(not(target_family = "wasm"))]
pub const fn sensitivity() -> f32 {
    BASE_SENSITIVITY
}
