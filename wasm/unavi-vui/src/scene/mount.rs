use wired_prelude::prelude::*;

use crate::scene::viewer;

/// Which way a surface is measured off the viewer when it is placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Bearing {
    /// Ahead on the ground plane, standing upright however the viewer was
    /// looking. A surface you walk up to, and one that never ends up
    /// underfoot or overhead.
    #[default]
    Level,
    /// Along the line of sight, turned to face the viewer. A surface summoned
    /// to where the eye already is, rather than one the eye has to find.
    Sight,
}

/// Where a surface stands, measured from the viewer at the moment it first
/// draws. It stays there afterwards: a menu that follows you around is a menu
/// you cannot walk up to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mount {
    /// Metres ahead of the viewer, along whichever direction `bearing` names.
    pub distance: f32,
    /// Metres above the eye, usually negative so a surface sits below it.
    pub height:   f32,
    /// Sideways and vertical shift in the mounted frame, so surfaces sharing
    /// one facing do not sit on top of each other.
    pub offset:   Vec2,
    pub bearing:  Bearing,
}

impl Mount {
    #[must_use]
    pub const fn ahead(distance: f32, height: f32) -> Self {
        Self {
            distance,
            height,
            offset: Vec2::ZERO,
            bearing: Bearing::Level,
        }
    }

    #[must_use]
    pub const fn beside(self, offset: Vec2) -> Self {
        Self { offset, ..self }
    }

    pub(crate) fn anchor(self, eye: &Transform) -> Transform {
        let (along, rotation) = match self.bearing {
            Bearing::Level => {
                let facing = viewer::facing(eye);
                (facing, viewer::yaw_only(facing))
            }
            Bearing::Sight => (eye.rotation * Vec3::new(0.0, 0.0, -1.0), eye.rotation),
        };
        Transform {
            translation: eye.translation
                + along * self.distance
                + Vec3::new(0.0, self.height, 0.0)
                + rotation * Vec3::new(self.offset.x, self.offset.y, 0.0),
            rotation,
            scale: Vec3::ONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DISTANCE: f32 = 1.0;

    fn pitch(radians: f32) -> Quat {
        Quat::new((radians * 0.5).sin(), 0.0, 0.0, (radians * 0.5).cos())
    }

    fn yaw(radians: f32) -> Quat {
        Quat::new(0.0, (radians * 0.5).sin(), 0.0, (radians * 0.5).cos())
    }

    /// Looking steeply down, which is where the two bearings part company.
    fn stooping() -> Transform {
        Transform {
            translation: Vec3::new(0.0, 1.6, 0.0),
            rotation:    pitch(-0.9),
            scale:       Vec3::ONE,
        }
    }

    #[test]
    fn a_level_mount_stands_at_eye_height_however_the_viewer_was_looking() {
        let eye = stooping();
        let anchor = Mount::ahead(DISTANCE, 0.0).anchor(&eye);
        assert!(
            (anchor.translation.y - eye.translation.y).abs() < 1.0e-5,
            "a surface you walk up to must not end up underfoot"
        );
    }

    #[test]
    fn a_sight_mount_stands_where_the_viewer_is_looking() {
        let eye = stooping();
        let mount = Mount {
            bearing: Bearing::Sight,
            ..Mount::ahead(DISTANCE, 0.0)
        };
        let anchor = mount.anchor(&eye);
        assert!(
            anchor.translation.y < eye.translation.y - 0.5,
            "looking down should summon it down there, not out at eye height"
        );
    }

    #[test]
    fn both_bearings_stand_the_same_distance_off() {
        let eye = stooping();
        let level = Mount::ahead(DISTANCE, 0.0).anchor(&eye);
        let sight = Mount {
            bearing: Bearing::Sight,
            ..Mount::ahead(DISTANCE, 0.0)
        }
        .anchor(&eye);

        for anchor in [level, sight] {
            let reach = (anchor.translation - eye.translation).length();
            assert!(
                (reach - DISTANCE).abs() < 1.0e-5,
                "distance means distance, whichever way it is measured"
            );
        }
    }

    #[test]
    fn looking_level_makes_the_two_bearings_agree() {
        let eye = Transform {
            translation: Vec3::new(2.0, 1.6, -3.0),
            rotation:    yaw(0.7),
            scale:       Vec3::ONE,
        };
        let level = Mount::ahead(DISTANCE, 0.0).anchor(&eye);
        let sight = Mount {
            bearing: Bearing::Sight,
            ..Mount::ahead(DISTANCE, 0.0)
        }
        .anchor(&eye);
        assert!((level.translation - sight.translation).length() < 1.0e-5);
    }
}
