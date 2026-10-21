//! Where a dragged element settles: the space's grid and 15° turn steps, and
//! the stick flicks that step it.

use std::f32::consts::PI;

use wired_guest::math::Vec3;

/// Metres between grid lines.
pub const GRID: f32 = 0.25;
/// Radians per turn step: 15°.
pub const TURN: f32 = PI / 12.0;

/// `at`, moved to the nearest grid line on the ground plane. Height is left
/// alone, so an element keeps resting where it rests.
#[must_use]
pub fn to_grid(at: Vec3) -> Vec3 {
    Vec3::new(
        (at.x / GRID).round() * GRID,
        at.y,
        (at.z / GRID).round() * GRID,
    )
}

/// Stick travel a flick must pass.
const FLICK: f32 = 0.7;
/// Stick travel it must come back under before the next flick counts.
const REARM: f32 = 0.3;

/// A direction a stick was flicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flick {
    Left,
    Right,
    Up,
    Down,
}

/// Turns a stick's travel into discrete flicks, one per push out.
#[derive(Clone, Copy, Debug, Default)]
pub struct Flicks {
    /// Whether the stick has come back since the last flick.
    spent: bool,
}

impl Flicks {
    /// Reads the stick at `x`, `y`, answering a flick the moment it passes
    /// [`FLICK`] along one axis.
    pub fn read(&mut self, x: f32, y: f32) -> Option<Flick> {
        if x.abs().max(y.abs()) < REARM {
            self.spent = false;
            return None;
        }
        if self.spent {
            return None;
        }
        let flick = if x.abs() >= y.abs() {
            (x.abs() > FLICK).then_some(if x > 0.0 { Flick::Right } else { Flick::Left })
        } else {
            (y.abs() > FLICK).then_some(if y > 0.0 { Flick::Up } else { Flick::Down })
        };
        if flick.is_some() {
            self.spent = true;
        }
        flick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_lands_on_the_nearest_grid_line_at_its_own_height() {
        let at = to_grid(Vec3::new(0.13, -0.12, 1.4));
        assert!((at - Vec3::new(0.25, -0.12, 1.5)).length() < 1.0e-6, "{at}");
    }

    #[test]
    fn one_push_out_is_one_flick() {
        let mut flicks = Flicks::default();
        assert_eq!(flicks.read(0.9, 0.1), Some(Flick::Right));
        assert_eq!(flicks.read(1.0, 0.0), None, "held out is not another flick");
        assert_eq!(flicks.read(0.1, 0.0), None);
        assert_eq!(flicks.read(-0.8, 0.0), Some(Flick::Left));
    }

    #[test]
    fn a_stick_drifting_between_the_thresholds_does_not_flick() {
        let mut flicks = Flicks::default();
        assert_eq!(flicks.read(0.5, 0.0), None);
        assert_eq!(flicks.read(0.0, -0.9), Some(Flick::Down));
    }
}
