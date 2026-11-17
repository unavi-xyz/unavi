//! How a pane floats: a weightless body the host's grab carries like any
//! other, which once let go coasts to a stop, holds where it stopped, and
//! turns upright to face the viewer.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use crate::wired::input::types::PointerKind;

/// Metres per second under which a let-go pane counts as stopped, and over
/// which a held-still one counts as pushed off again.
const STOPPED: f32 = 0.03;
const PUSHED: f32 = 0.4;
/// Gap fraction closed per second toward where it rests, and toward facing
/// the viewer.
const SETTLE: f32 = 3.0;
const TURN: f32 = 2.5;
/// Radians under which it counts as facing the viewer.
const FACING: f32 = 0.01;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Float {
    /// The pointer whose grip carries it.
    pub held_by: Option<PointerKind>,
    /// Where it stopped, which it holds against small knocks.
    rest:        Option<Vec3>,
}

/// The velocities a free pane is set to: `None` leaves one as it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Steer {
    pub linear:  Option<Vec3>,
    pub angular: Vec3,
}

impl Float {
    /// Holds it where it is from now on, as after a move the script made,
    /// which takes it out of any hand.
    pub const fn rest_at(&mut self, at: Vec3) {
        self.held_by = None;
        self.rest = Some(at);
    }

    pub const fn grab(&mut self, pointer: PointerKind) {
        self.held_by = Some(pointer);
        self.rest = None;
    }

    /// Lets go, if `pointer` is what held it. It coasts on from here.
    pub fn release(&mut self, pointer: PointerKind) {
        if self.held_by == Some(pointer) {
            self.held_by = None;
        }
    }

    /// What to set a pane posed `body` and moving at `velocity` to, so it
    /// floats facing `eye`. `None` while a hand holds it, since the grab
    /// drives it then.
    pub fn steer(&mut self, body: &Transform, velocity: Vec3, eye: Vec3) -> Option<Steer> {
        if self.held_by.is_some() {
            return None;
        }
        // Moving fast, something else drives it: a grab the pane did not
        // hear, a beam, a shove. It is left to that until it slows.
        let speed = velocity.length();
        if speed > PUSHED {
            self.rest = None;
            return None;
        }
        if self.rest.is_none() && speed < STOPPED {
            self.rest = Some(body.translation);
        }
        let linear = self.rest.map(|rest| (rest - body.translation) * SETTLE);
        Some(Steer {
            linear,
            angular: turn_toward(body.rotation, facing(body.translation, eye)),
        })
    }
}

/// Upright, its front (+Z) toward `eye`.
#[must_use]
pub fn facing(at: Vec3, eye: Vec3) -> Quat {
    let toward = eye - at;
    Quat::from_rotation_y(toward.x.atan2(toward.z))
}

/// The angular velocity that closes [`TURN`] of the way from `from` to `to`
/// each second, the short way round.
fn turn_toward(from: Quat, to: Quat) -> Vec3 {
    let mut diff = glam::Quat::from(to) * glam::Quat::from(from).inverse();
    if diff.w < 0.0 {
        diff = -diff;
    }
    let (axis, angle) = diff.normalize().to_axis_angle();
    if angle < FACING || !axis.is_finite() {
        return Vec3::ZERO;
    }
    axis * angle * TURN
}

#[cfg(test)]
mod tests {
    use super::*;

    const EYE: Vec3 = Vec3::new(0.0, 1.6, 0.0);

    fn at(translation: Vec3) -> Transform {
        Transform {
            translation,
            rotation: facing(translation, EYE),
            scale: Vec3::ONE,
        }
    }

    #[test]
    fn a_held_pane_is_left_to_the_hand() {
        let mut float = Float::default();
        float.grab(PointerKind::RightHand);
        assert_eq!(float.steer(&at(Vec3::ZERO), Vec3::ZERO, EYE), None);
    }

    #[test]
    fn a_let_go_pane_coasts_then_holds_where_it_stopped() {
        let mut float = Float::default();
        float.grab(PointerKind::LeftHand);
        float.release(PointerKind::LeftHand);
        let moving = float.steer(&at(Vec3::new(0.0, 1.5, -1.0)), Vec3::X, EYE);
        assert_eq!(moving.and_then(|steer| steer.linear), None);
        let stopped = Vec3::new(0.3, 1.5, -1.0);
        let _ = float.steer(&at(stopped), Vec3::ZERO, EYE);
        let knocked = float.steer(&at(stopped + Vec3::Y * 0.01), Vec3::ZERO, EYE);
        let Some(Some(back)) = knocked.map(|steer| steer.linear) else {
            panic!("it holds where it stopped");
        };
        assert!(back.y < 0.0, "{back}");
    }

    #[test]
    fn a_pane_something_else_moves_fast_is_left_to_it() {
        let mut float = Float::default();
        float.rest_at(Vec3::ZERO);
        assert_eq!(float.steer(&at(Vec3::X), Vec3::X * 2.0, EYE), None);
    }

    #[test]
    fn moving_a_held_pane_takes_it_out_of_the_hand() {
        let mut float = Float::default();
        float.grab(PointerKind::RightHand);
        float.rest_at(Vec3::ZERO);
        assert!(float.steer(&at(Vec3::ZERO), Vec3::ZERO, EYE).is_some());
    }

    #[test]
    fn another_hand_letting_go_does_not_drop_it() {
        let mut float = Float::default();
        float.grab(PointerKind::RightHand);
        float.release(PointerKind::LeftHand);
        assert_eq!(float.held_by, Some(PointerKind::RightHand));
    }

    #[test]
    fn a_pane_facing_away_turns_toward_the_eye_about_the_vertical() {
        let mut float = Float::default();
        let body = Transform {
            rotation: Quat::from_rotation_y(1.0),
            ..at(Vec3::new(0.0, 1.6, -1.0))
        };
        let Some(steer) = float.steer(&body, Vec3::ZERO, EYE) else {
            panic!("a free pane is steered");
        };
        assert!(
            steer.angular.y < 0.0 && steer.angular.x.abs() < 1.0e-4,
            "{}",
            steer.angular
        );
    }

    #[test]
    fn a_pane_facing_the_eye_is_not_turned() {
        let mut float = Float::default();
        let Some(steer) = float.steer(&at(Vec3::new(0.5, 1.6, -1.0)), Vec3::ZERO, EYE) else {
            panic!("a free pane is steered");
        };
        assert_eq!(steer.angular, Vec3::ZERO);
    }
}
