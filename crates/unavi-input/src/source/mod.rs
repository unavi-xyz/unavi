use bevy::prelude::Vec2;

pub mod gamepad;
pub mod keyboard;
pub mod mouse;
#[cfg(not(target_family = "wasm"))] pub mod xr;

/// Rescales so a stick just past the deadzone reads as barely moved rather
/// than jumping to the deadzone's own value.
#[must_use]
pub fn deadzone(raw: Vec2, deadzone: f32) -> Vec2 {
    let length = raw.length();
    if length <= deadzone {
        return Vec2::ZERO;
    }
    let scaled = ((length - deadzone) / (1.0 - deadzone)).min(1.0);
    raw / length * scaled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drift_inside_the_deadzone_is_not_movement() {
        assert_eq!(deadzone(Vec2::new(0.1, 0.0), 0.15), Vec2::ZERO);
    }

    #[test]
    fn a_stick_leaving_the_deadzone_starts_from_nothing() {
        let value = deadzone(Vec2::new(0.16, 0.0), 0.15);
        assert!(value.x > 0.0 && value.x < 0.02, "got {}", value.x);
    }

    #[test]
    fn a_stick_pushed_all_the_way_still_reaches_full() {
        let value = deadzone(Vec2::new(0.0, 1.0), 0.15);
        assert!((value.length() - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn a_stick_keeps_its_direction_through_the_rescale() {
        let raw = Vec2::new(0.6, 0.8);
        let value = deadzone(raw, 0.15);
        assert!(value.normalize().distance(raw.normalize()) < 1.0e-5);
    }
}
