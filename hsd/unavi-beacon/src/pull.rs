//! The draw of a socket on a beacon let go near it: around whatever backs
//! the socket, up over it, then down into it, turning to the nearest way a
//! cube can sit square.

use std::f32::consts::FRAC_PI_2;

use wired_guest::{
    beacon::{
        BEACON_SIZE,
        Backing,
    },
    math::{
        Quat,
        Transform,
        Vec3,
    },
};

use crate::float::Drive;

/// Metres from a socket within which a beacon lying still is drawn in.
pub const NEAR_RADIUS: f32 = 0.35;
/// Seconds after a release during which a socket draws from its full radius.
pub const THROW_WINDOW: f32 = 2.0;
/// Metres a pull must close on its seat within [`STALL_SECS`] to keep going.
pub const PROGRESS: f32 = 0.01;
pub const STALL_SECS: f32 = 1.0;

/// Metres per second.
const MAX_SPEED: f32 = 3.5;
/// Per second of distance, the speed asked for before [`MAX_SPEED`].
const GAIN: f32 = 7.0;
/// Per second, how fast the body's velocity turns to the one asked for.
const BLEND: f32 = 12.0;
const TURN: f32 = 10.0;
/// Height over the seat a beacon comes in at, clearing the socket's rim.
const APPROACH_LIFT: f32 = 0.09;
/// Horizontal metres from the seat within which a beacon descends.
const OVER_SEAT: f32 = 0.015;
/// Metres a beacon passing a socket's backing keeps clear of it.
const CLEARANCE: f32 = BEACON_SIZE / 2.0 + 0.05;
/// Metres from the seat, and metres per second, under which a beacon rests.
const REST_DISTANCE: f32 = 0.008;
const REST_SPEED: f32 = 0.06;

/// Whether a beacon at `now`, moving at `speed`, has come to rest in `seat`.
pub fn at_rest(now: Vec3, seat: Vec3, speed: f32) -> bool {
    now.distance(seat) < REST_DISTANCE && speed < REST_SPEED
}

/// The velocities that carry a body at `now`, moving at `velocity`, toward
/// `seat` over `dt`, going around `backing` first when it is in the way.
pub fn drive(now: Transform, seat: Transform, backing: Backing, velocity: Vec3, dt: f32) -> Drive {
    let to_seat = seat.translation - now.translation;
    let across = Vec3::new(to_seat.x, 0.0, to_seat.z).length();
    let local = seat.rotation.inverse() * (now.translation - seat.translation);
    let around = backing.half_depth + CLEARANCE;
    let side = if local.z < 0.0 { -1.0 } else { 1.0 };
    // Beside the backing: first step out past its depth where the beacon is,
    // then run along its side to the seat's line, so no leg cuts through it.
    let aim = if local.x > backing.from - CLEARANCE {
        let x = if local.z.abs() < around * 0.9 {
            local.x
        } else {
            0.0
        };
        seat.translation + seat.rotation * Vec3::new(x, APPROACH_LIFT, side * around)
    } else if across > OVER_SEAT {
        seat.translation + Vec3::Y * APPROACH_LIFT
    } else {
        seat.translation
    };
    let toward = aim - now.translation;
    let distance = toward.length();
    let wanted = if distance > f32::EPSILON {
        toward / distance * (distance * GAIN).min(MAX_SPEED)
    } else {
        Vec3::ZERO
    };
    let linear = velocity.lerp(wanted, (dt * BLEND).min(1.0));

    let target = squared_to(seat.rotation, now.rotation);
    let mut diff = target * now.rotation.inverse();
    if diff.w < 0.0 {
        diff = -diff;
    }
    let (axis, angle) = diff.normalize().to_axis_angle();
    let angular = if angle.is_finite() {
        axis * angle * TURN
    } else {
        Vec3::ZERO
    };

    Drive { linear, angular }
}

/// Of the 24 rotations that leave a cube looking the same in `frame`, the one
/// nearest `now`.
fn squared_to(frame: Quat, now: Quat) -> Quat {
    let frame = glam::Quat::from(frame);
    let now = glam::Quat::from(now);
    let ups = [
        glam::Quat::IDENTITY,
        glam::Quat::from_rotation_x(FRAC_PI_2),
        glam::Quat::from_rotation_x(2.0 * FRAC_PI_2),
        glam::Quat::from_rotation_x(3.0 * FRAC_PI_2),
        glam::Quat::from_rotation_z(FRAC_PI_2),
        glam::Quat::from_rotation_z(3.0 * FRAC_PI_2),
    ];
    let mut best = frame;
    let mut best_dot = -1.0_f32;
    for up in ups {
        for turn in 0..4 {
            let candidate = frame * up * glam::Quat::from_rotation_y(turn as f32 * FRAC_PI_2);
            let dot = candidate.dot(now).abs();
            if dot > best_dot {
                best_dot = dot;
                best = candidate;
            }
        }
    }
    Quat::from(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A backing nothing reaches.
    const OPEN: Backing = Backing {
        from:       10.0,
        half_depth: 0.0,
    };

    fn at(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_translation(Vec3::new(x, y, z))
    }

    #[test]
    fn a_beacon_beside_the_socket_comes_in_over_it() {
        let seat = at(0.0, 1.0, 0.0);
        let drive = drive(at(-0.5, 1.0, 0.0), seat, OPEN, Vec3::ZERO, 1.0);
        assert!(drive.linear.x > 0.0, "not drawn toward the socket");
        assert!(
            drive.linear.y > 0.0,
            "came in level, into the socket's rim: {:?}",
            drive.linear
        );
    }

    #[test]
    fn a_beacon_over_the_socket_comes_straight_down() {
        let seat = at(0.0, 1.0, 0.0);
        let drive = drive(at(0.0, 1.09, 0.0), seat, OPEN, Vec3::ZERO, 1.0);
        assert!(drive.linear.y < 0.0);
        assert!(drive.linear.x.abs() < 1.0e-6 && drive.linear.z.abs() < 1.0e-6);
    }

    #[test]
    fn a_beacon_thrown_from_the_doorway_goes_around_the_pillar_into_the_socket() {
        // A pillar beside the socket, grown by half a beacon so the box holds
        // every centre that would touch it.
        let backing = Backing {
            from:       0.19,
            half_depth: 0.3,
        };
        let half = BEACON_SIZE / 2.0;
        let pillar_x = (backing.from - half)..(backing.from + 0.36 + half);
        let pillar_z = backing.half_depth + half;
        let seat = at(0.0, 0.0, 0.0);
        let mut position = Vec3::new(0.9, 0.0, 0.05);
        let mut velocity = Vec3::ZERO;
        let dt = 1.0 / 64.0;
        for _ in 0..640 {
            let d = drive(
                Transform::from_translation(position),
                seat,
                backing,
                velocity,
                dt,
            );
            velocity = d.linear;
            position += velocity * dt;
            assert!(
                !(pillar_x.contains(&position.x) && position.z.abs() < pillar_z),
                "drove into the pillar at {position:?}"
            );
        }
        assert!(
            position.length() < REST_DISTANCE,
            "stopped at {position:?}, short of the seat"
        );
    }

    #[test]
    fn the_draw_never_outruns_its_cap() {
        let seat = at(0.0, 0.0, 0.0);
        let drive = drive(at(-1.2, 0.0, 0.0), seat, OPEN, Vec3::ZERO, 1.0);
        assert!(drive.linear.length() <= MAX_SPEED + 1.0e-4);
    }

    #[test]
    fn a_beacon_turns_the_short_way_to_sit_square() {
        let now = Quat::from_rotation_y(80.0_f32.to_radians());
        let target = squared_to(Quat::IDENTITY, now);
        let angle = glam::Quat::from(target)
            .angle_between(glam::Quat::from(now))
            .to_degrees();
        assert!(angle < 11.0, "turned {angle} degrees rather than 10");
    }

    #[test]
    fn a_body_in_its_seat_and_slow_is_at_rest() {
        assert!(at_rest(Vec3::new(0.0, 0.004, 0.0), Vec3::ZERO, 0.01));
        assert!(!at_rest(Vec3::new(0.0, 0.004, 0.0), Vec3::ZERO, 0.5));
        assert!(!at_rest(Vec3::new(0.0, 0.05, 0.0), Vec3::ZERO, 0.0));
    }
}
