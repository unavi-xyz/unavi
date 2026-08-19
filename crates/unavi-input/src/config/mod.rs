use std::ops::RangeInclusive;

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

impl Tuning {
    /// Every value held to the domain the readers assume, because a hand-edited
    /// file reaches them unchecked: a deadzone of one divides by zero and sends
    /// a NaN move axis into the solver, and a press threshold of zero holds
    /// every button down forever.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let sane = Self::default();

        hold(&mut self.look_sensitivity, sane.look_sensitivity, 0.0..=1.0);
        hold(
            &mut self.stick_deadzone,
            sane.stick_deadzone,
            0.0..=MAX_DEADZONE,
        );
        hold(&mut self.move_threshold, sane.move_threshold, 0.0..=1.0);
        hold(&mut self.turn_threshold, sane.turn_threshold, 0.0..=1.0);
        hold(
            &mut self.snap_turn_degrees,
            sane.snap_turn_degrees,
            0.0..=180.0,
        );
        hold(
            &mut self.press_threshold,
            sane.press_threshold,
            MIN_PRESS_THRESHOLD..=1.0,
        );
        hold(
            &mut self.pointer_reach,
            sane.pointer_reach,
            0.0..=MAX_POINTER_REACH,
        );
        hold(
            &mut self.look_degrees_per_second,
            sane.look_degrees_per_second,
            0.0..=MAX_TURN_RATE,
        );
        hold(
            &mut self.smooth_turn_degrees_per_second,
            sane.smooth_turn_degrees_per_second,
            0.0..=MAX_TURN_RATE,
        );

        self
    }
}

/// Short of one, so rescaling a stick past the deadzone never divides by zero.
const MAX_DEADZONE: f32 = 0.95;
/// Above zero, so a button nobody is touching is not held.
const MIN_PRESS_THRESHOLD: f32 = 1.0e-3;
const MAX_POINTER_REACH: f32 = 1000.0;
const MAX_TURN_RATE: f32 = 3600.0;

/// `clamp` alone would carry a NaN through, and NaN is the one value with no
/// place anywhere in a range.
const fn hold(value: &mut f32, fallback: f32, range: RangeInclusive<f32>) {
    *value = if value.is_finite() {
        value.clamp(*range.start(), *range.end())
    } else {
        fallback
    };
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
    use crate::source::deadzone;

    #[test]
    fn a_config_missing_everything_still_loads() {
        let config = file::parse("()").expect("parse");
        assert!(
            (config.tuning.pointer_reach - Tuning::default().pointer_reach).abs() < f32::EPSILON
        );
        assert!(!config.bindings.movement.dpads.is_empty());
    }

    #[test]
    fn a_deadzone_of_one_does_not_reach_the_readers() {
        let config = file::parse("(tuning: (stick_deadzone: 1.0))").expect("parse");
        let value = deadzone(Vec2::new(0.99, 0.0), config.tuning.stick_deadzone);
        assert!(
            value.is_finite(),
            "a hand-edited deadzone must not divide by zero into the move axis"
        );
    }

    #[test]
    fn a_threshold_of_zero_does_not_hold_every_button_down() {
        let config = file::parse("(tuning: (press_threshold: 0.0))").expect("parse");
        assert!(config.tuning.press_threshold > 0.0);
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
