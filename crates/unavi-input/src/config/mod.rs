use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

pub mod bindings;
pub mod file;
pub mod patch;

/// The bindings and tuning in force, a config file resolved over the defaults.
#[derive(Resource, Clone, Debug, Default)]
pub struct InputConfig {
    pub bindings: bindings::Bindings,
    pub tuning:   Tuning,
}

/// The feel of the input, as opposed to what is bound to what.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Tuning {
    /// Radians of view turn per unit of mouse travel.
    pub look_sensitivity:               f32,
    /// How fast a look stick turns the desktop view.
    pub look_degrees_per_second:        f32,
    /// Stick travel below this reads as centre, covering drift.
    pub stick_deadzone:                 f32,
    /// Stick travel below this reads as standing still. Above it the agent
    /// walks at the stick's own pace.
    pub move_threshold:                 f32,
    /// How hard a bound button must be held to count as pressed. Only an
    /// analogue binding can land under it.
    pub press_threshold:                f32,
    /// How far a pointer can reach, in metres.
    pub pointer_reach:                  f32,
    pub smooth_turn:                    bool,
    pub snap_turn_degrees:              f32,
    pub smooth_turn_degrees_per_second: f32,
    /// Stick travel that turns the VR view.
    pub turn_threshold:                 f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            look_sensitivity:               0.002,
            look_degrees_per_second:        120.0,
            stick_deadzone:                 0.15,
            move_threshold:                 0.2,
            press_threshold:                0.5,
            pointer_reach:                  2.5,
            smooth_turn:                    false,
            snap_turn_degrees:              30.0,
            smooth_turn_degrees_per_second: 120.0,
            turn_threshold:                 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_config_missing_everything_still_loads() {
        let config = file::parse("()").expect("parse");
        assert!(
            (config.tuning.pointer_reach - Tuning::default().pointer_reach).abs() < f32::EPSILON
        );
        assert!(!config.bindings.movement.dpads.is_empty());
    }

    #[test]
    fn a_config_naming_one_field_keeps_the_defaults_for_the_rest() {
        let config = file::parse("(tuning: (look_sensitivity: 0.5))").expect("parse");
        assert!((config.tuning.look_sensitivity - 0.5).abs() < f32::EPSILON);
        assert!(
            (config.tuning.move_threshold - Tuning::default().move_threshold).abs() < f32::EPSILON
        );
    }
}
