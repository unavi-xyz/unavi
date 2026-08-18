use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
};

use crate::{
    action::ActionState,
    config::InputConfig,
};

/// Pointer lock reports deltas in different units per platform, so the
/// configured sensitivity means the same thing everywhere.
#[cfg(target_family = "wasm")]
fn platform_scale() -> f32 {
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
const fn platform_scale() -> f32 {
    1.0
}

pub fn read(
    motion: Res<AccumulatedMouseMotion>,
    buttons: Res<ButtonInput<MouseButton>>,
    config: Res<InputConfig>,
    mut state: ResMut<ActionState>,
) {
    // Screen space runs down; looking does not.
    let delta = Vec2::new(motion.delta.x, -motion.delta.y) * platform_scale();

    if delta != Vec2::ZERO {
        for (action, _) in config.bindings.axes().filter(|(_, b)| b.mouse_motion) {
            state.accumulate_delta(action, delta);
        }
    }

    for (action, binding) in config.bindings.buttons() {
        if binding.mouse.iter().any(|button| buttons.pressed(*button)) {
            state.press(action, 1.0);
        }
    }
}
