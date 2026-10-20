//! The hover a beacon keeps after it is let go: hung where it was released,
//! bobbing and turning, until a timeout or a hand takes it.

use std::f32::consts::TAU;

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

/// Seconds a planted beacon hangs in the air before it falls.
pub const FLOAT_SECS: f32 = 8.0;

/// Peak height of the bob above the release point, in meters.
const BOB_AMPLITUDE: f32 = 0.03;
const BOB_PERIOD: f32 = 2.5;
/// Radians per second about the vertical axis.
const TURN_RATE: f32 = 0.6;
/// How hard the body is pulled back onto its hover path, per second of error.
const FOLLOW: f32 = 8.0;

/// How far the hover has moved a beacon from where it was released.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Offset {
    pub lift: f32,
    pub yaw:  f32,
}

/// The velocities that keep a body on its hover path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drive {
    pub linear:  Vec3,
    pub angular: Vec3,
}

/// Whether a beacon is still hovering, and for how long it has been.
#[derive(Clone, Copy, Debug)]
pub struct Float {
    elapsed: f32,
    over:    bool,
}

impl Float {
    /// A hover that began `age` seconds ago, already over if that is more
    /// than [`FLOAT_SECS`].
    pub fn at_age(age: f32) -> Self {
        Self {
            elapsed: age.clamp(0.0, FLOAT_SECS),
            over:    age >= FLOAT_SECS,
        }
    }

    /// A beacon with no hover left to do.
    pub const fn over() -> Self {
        Self {
            elapsed: FLOAT_SECS,
            over:    true,
        }
    }

    pub const fn is_floating(self) -> bool {
        !self.over
    }

    /// Advances the hover by `dt`. A grab ends it at once, as does running
    /// out of [`FLOAT_SECS`]. Returns true on the step that ends it, so the
    /// caller makes the beacon fall exactly once.
    pub fn step(&mut self, dt: f32, grabbed: bool) -> bool {
        if self.over {
            return false;
        }
        self.elapsed += dt;
        if grabbed || self.elapsed >= FLOAT_SECS {
            self.over = true;
            return true;
        }
        false
    }

    /// Ends the hover early, as a socket drawing the beacon in does.
    pub const fn end(&mut self) {
        self.elapsed = FLOAT_SECS;
        self.over = true;
    }

    /// The pose offset at this point of the hover. Zero at the start, so a
    /// beacon does not jump when it begins to float.
    pub fn offset(self) -> Offset {
        Offset {
            lift: BOB_AMPLITUDE * (TAU * self.elapsed / BOB_PERIOD).sin(),
            yaw:  TURN_RATE * self.elapsed,
        }
    }

    /// The velocities that carry a body now at `now` along the hover about
    /// `rest`: the path's own rate, plus a pull back onto the path so
    /// collisions and the physics step do not let the body wander off it.
    pub fn drive(self, rest: Transform, now: Transform) -> Drive {
        let offset = self.offset();
        let lift_rate =
            BOB_AMPLITUDE * (TAU / BOB_PERIOD) * (TAU * self.elapsed / BOB_PERIOD).cos();

        let target = rest.translation + Vec3::Y * offset.lift;
        let linear = (target - now.translation) * FOLLOW + Vec3::Y * lift_rate;

        let target_rotation = Quat::from_rotation_y(offset.yaw) * rest.rotation;
        let mut diff = target_rotation * now.rotation.inverse();
        // Quaternions double-cover rotations; take the short way round.
        if diff.w < 0.0 {
            diff = -diff;
        }
        let (axis, angle) = diff.normalize().to_axis_angle();
        let angular = axis * angle * FOLLOW + Vec3::Y * TURN_RATE;

        Drive { linear, angular }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_starts_floating_at_rest() {
        let float = Float::at_age(0.0);
        assert!(float.is_floating());
        assert_eq!(
            float.offset(),
            Offset {
                lift: 0.0,
                yaw:  0.0,
            }
        );
    }

    #[test]
    fn it_keeps_floating_until_the_timeout() {
        let mut float = Float::at_age(0.0);
        assert!(!float.step(FLOAT_SECS - 0.5, false));
        assert!(float.is_floating());
        assert!(float.step(0.5, false), "the timeout did not release");
        assert!(!float.is_floating());
    }

    #[test]
    fn a_grab_releases_at_once() {
        let mut float = Float::at_age(0.0);
        assert!(float.step(0.1, true), "a grab did not release");
        assert!(!float.is_floating());
    }

    #[test]
    fn it_releases_only_once() {
        let mut float = Float::at_age(0.0);
        assert!(float.step(FLOAT_SECS, false));
        assert!(!float.step(0.1, false));
        assert!(!float.step(0.1, true));
    }

    #[test]
    fn a_late_joiner_picks_up_mid_hover() {
        let mut float = Float::at_age(6.0);
        assert!(float.is_floating());
        assert!(float.step(2.0, false), "the remaining time did not release");
    }

    #[test]
    fn a_beacon_that_already_fell_does_not_float_again() {
        assert!(!Float::at_age(FLOAT_SECS + 30.0).is_floating());
        assert!(!Float::over().is_floating());
    }

    #[test]
    fn the_bob_stays_within_its_amplitude() {
        let mut float = Float::at_age(0.0);
        let mut peak = 0.0_f32;
        while float.is_floating() {
            float.step(0.05, false);
            peak = peak.max(float.offset().lift.abs());
        }
        assert!(peak <= BOB_AMPLITUDE + 1.0e-6, "bobbed {peak} m");
        assert!(peak > BOB_AMPLITUDE * 0.9, "bobbed only {peak} m");
    }

    #[test]
    fn it_turns_steadily() {
        let mut float = Float::at_age(0.0);
        float.step(2.0, false);
        let expected = TURN_RATE * 2.0;
        assert!((float.offset().yaw - expected).abs() < 1.0e-5);
    }

    #[test]
    fn a_body_on_its_path_is_only_given_the_path_rate() {
        let rest = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let drive = Float::at_age(0.0).drive(rest, rest);
        assert!(drive.linear.x.abs() < 1.0e-6 && drive.linear.z.abs() < 1.0e-6);
        assert!(drive.linear.y > 0.0, "the bob starts upward");
        assert!(drive.angular.x.abs() < 1.0e-6 && drive.angular.z.abs() < 1.0e-6);
        assert!((drive.angular.y - TURN_RATE).abs() < 1.0e-5);
    }

    #[test]
    fn a_body_pushed_off_its_path_is_pulled_back() {
        let rest = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let pushed = Transform::from_translation(Vec3::new(1.5, 2.0, 3.0));
        let drive = Float::at_age(0.0).drive(rest, pushed);
        assert!(drive.linear.x < 0.0, "pushed +x, driven {}", drive.linear.x);
    }
}
