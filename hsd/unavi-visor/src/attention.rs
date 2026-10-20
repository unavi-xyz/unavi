//! Which things in the world the viewer is attending to.
//!
//! A thing is attended when it is close and the viewer is looking at its
//! placard. An attended thing keeps its place until the viewer has clearly
//! moved on, so a gaze drifting across the edge of the cone does not make a
//! placard flicker.

use std::f32::consts::PI;

use wired_guest::math::Vec3;

/// Radians between the gaze and the placard within which a thing may become
/// attended.
pub const ENTER_ANGLE: f32 = 20.0 * PI / 180.0;

/// Radians past which an attended thing is let go.
pub const KEEP_ANGLE: f32 = 30.0 * PI / 180.0;

/// The most things of one kind attended at once.
pub const CAP: usize = 3;

/// How far away a kind of thing may be attended, in metres from the eye to
/// its centre.
#[derive(Debug, Clone, Copy)]
pub struct Reach {
    /// Within which it may become attended.
    pub enter: f32,
    /// Past which an attended one is let go.
    pub keep:  f32,
}

/// Portals, which are looked at to read where they go.
pub const PORTALS: Reach = Reach {
    enter: 6.0,
    keep:  7.0,
};

/// People, whose names are read from across a room.
pub const PEOPLE: Reach = Reach {
    enter: 10.0,
    keep:  12.0,
};

/// A thing that might be attended.
#[derive(Debug, Clone, Copy)]
pub struct Candidate<K> {
    pub key:    K,
    /// Where the thing is.
    pub centre: Vec3,
    /// Where its placard would stand.
    pub anchor: Vec3,
}

/// The candidates attended from `eye` looking along `gaze`, at most [`CAP`]
/// of them. `held` are those attended last time, which are held to the looser
/// limits and come ahead of any newcomer, so a crowd at the cap cannot push
/// out a placard the viewer is already reading. Within each group the smallest
/// angle comes first.
impl Reach {
    #[must_use]
    pub fn attend<K: Copy + PartialEq>(
        self,
        eye: Vec3,
        gaze: Vec3,
        candidates: &[Candidate<K>],
        held: &[K],
    ) -> Vec<K> {
        let Some(gaze) = gaze.try_normalize() else {
            return Vec::new();
        };
        let mut attended: Vec<(bool, f32, K)> = candidates
            .iter()
            .filter_map(|candidate| {
                let distance = candidate.centre.distance(eye);
                let angle = angle_to(eye, gaze, candidate.anchor);
                let kept = held.contains(&candidate.key);
                let (range, cone) = if kept {
                    (self.keep, KEEP_ANGLE)
                } else {
                    (self.enter, ENTER_ANGLE)
                };
                (distance <= range && angle <= cone).then_some((kept, angle, candidate.key))
            })
            .collect();
        attended.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.total_cmp(&b.1)));
        attended.truncate(CAP);
        attended.into_iter().map(|(_, _, key)| key).collect()
    }
}

/// Radians between `gaze`, a unit vector, and the line from `eye` to `point`.
/// A point at the eye is straight ahead.
fn angle_to(eye: Vec3, gaze: Vec3, point: Vec3) -> f32 {
    (point - eye)
        .try_normalize()
        .map_or(0.0, |toward| gaze.dot(toward).clamp(-1.0, 1.0).acos())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EYE: Vec3 = Vec3::ZERO;
    const AHEAD: Vec3 = Vec3::NEG_Z;

    /// A portal `distance` metres out, `degrees` off the gaze to the right.
    fn portal(key: u8, distance: f32, degrees: f32) -> Candidate<u8> {
        let angle = degrees.to_radians();
        let at = Vec3::new(distance * angle.sin(), 0.0, -distance * angle.cos());
        Candidate {
            key,
            centre: at,
            anchor: at,
        }
    }

    #[test]
    fn a_near_portal_dead_ahead_is_attended() {
        assert_eq!(
            PORTALS.attend(EYE, AHEAD, &[portal(1, 4.0, 0.0)], &[]),
            vec![1]
        );
    }

    #[test]
    fn a_far_portal_is_not_attended_but_a_held_one_is_kept() {
        let far = [portal(1, 6.5, 0.0)];
        assert!(PORTALS.attend(EYE, AHEAD, &far, &[]).is_empty());
        assert_eq!(PORTALS.attend(EYE, AHEAD, &far, &[1]), vec![1]);
    }

    #[test]
    fn a_held_portal_goes_past_seven_metres() {
        assert!(
            PORTALS
                .attend(EYE, AHEAD, &[portal(1, 7.5, 0.0)], &[1])
                .is_empty()
        );
    }

    #[test]
    fn a_wide_portal_is_not_attended_but_a_held_one_is_kept() {
        let wide = [portal(1, 4.0, 25.0)];
        assert!(PORTALS.attend(EYE, AHEAD, &wide, &[]).is_empty());
        assert_eq!(PORTALS.attend(EYE, AHEAD, &wide, &[1]), vec![1]);
    }

    #[test]
    fn a_held_portal_goes_past_thirty_degrees() {
        assert!(
            PORTALS
                .attend(EYE, AHEAD, &[portal(1, 4.0, 35.0)], &[1])
                .is_empty()
        );
    }

    #[test]
    fn the_cone_edges_are_inclusive_of_what_is_just_inside() {
        assert_eq!(
            PORTALS.attend(EYE, AHEAD, &[portal(1, 5.9, 19.0)], &[]),
            vec![1]
        );
        assert!(
            PORTALS
                .attend(EYE, AHEAD, &[portal(1, 6.1, 19.0)], &[])
                .is_empty()
        );
        assert!(
            PORTALS
                .attend(EYE, AHEAD, &[portal(1, 5.9, 21.0)], &[])
                .is_empty()
        );
    }

    #[test]
    fn behind_the_viewer_is_never_attended() {
        assert!(
            PORTALS
                .attend(EYE, AHEAD, &[portal(1, 3.0, 180.0)], &[1])
                .is_empty()
        );
    }

    #[test]
    fn at_most_three_are_attended_smallest_angle_first() {
        let ring = [
            portal(1, 4.0, 15.0),
            portal(2, 4.0, 5.0),
            portal(3, 4.0, 10.0),
            portal(4, 4.0, 2.0),
            portal(5, 4.0, 18.0),
        ];
        assert_eq!(PORTALS.attend(EYE, AHEAD, &ring, &[]), vec![4, 2, 3]);
    }

    #[test]
    fn a_held_portal_is_not_pushed_out_by_closer_newcomers() {
        let crowd = [
            portal(1, 4.0, 1.0),
            portal(2, 4.0, 2.0),
            portal(3, 4.0, 3.0),
            portal(4, 4.0, 25.0),
        ];
        assert_eq!(PORTALS.attend(EYE, AHEAD, &crowd, &[4]), vec![4, 1, 2]);
    }

    #[test]
    fn the_angle_is_measured_to_the_placard_not_the_centre() {
        let tall = Candidate {
            key:    1_u8,
            centre: Vec3::new(0.0, 0.0, -4.0),
            anchor: Vec3::new(0.0, 4.0, -4.0),
        };
        assert!(PORTALS.attend(EYE, AHEAD, &[tall], &[]).is_empty());
    }

    #[test]
    fn people_are_attended_from_further_than_portals() {
        let across_the_room = [portal(1, 9.0, 0.0)];
        assert!(PORTALS.attend(EYE, AHEAD, &across_the_room, &[]).is_empty());
        assert_eq!(PEOPLE.attend(EYE, AHEAD, &across_the_room, &[]), vec![1]);
    }

    #[test]
    fn a_gaze_with_no_direction_attends_nothing() {
        assert!(
            PORTALS
                .attend(EYE, Vec3::ZERO, &[portal(1, 4.0, 0.0)], &[])
                .is_empty()
        );
    }

    #[test]
    fn the_eye_may_be_anywhere() {
        let eye = Vec3::new(10.0, 2.0, 5.0);
        let at = eye + Vec3::new(0.0, 0.0, -3.0);
        let candidate = Candidate {
            key:    9_u8,
            centre: at,
            anchor: at,
        };
        assert_eq!(PORTALS.attend(eye, AHEAD, &[candidate], &[]), vec![9]);
    }
}
