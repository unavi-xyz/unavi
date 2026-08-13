use wired_prelude::prelude::*;

use crate::scene::viewer;

/// Where a surface stands, measured from the viewer at the moment it first
/// draws. It stays there afterwards: a menu that follows you around is a menu
/// you cannot walk up to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mount {
    /// Metres ahead of the viewer, along their facing flattened to the ground.
    pub distance: f32,
    /// Metres above the eye, usually negative so a surface sits below it.
    pub height:   f32,
    /// Sideways and vertical shift in the mounted frame, so surfaces sharing
    /// one facing do not sit on top of each other.
    pub offset:   Vec2,
}

impl Mount {
    #[must_use]
    pub const fn ahead(distance: f32, height: f32) -> Self {
        Self {
            distance,
            height,
            offset: Vec2::ZERO,
        }
    }

    #[must_use]
    pub const fn beside(self, offset: Vec2) -> Self {
        Self { offset, ..self }
    }

    pub(crate) fn anchor(self, eye: &Transform) -> Transform {
        let facing = viewer::facing(eye);
        let rotation = viewer::yaw_only(facing);
        Transform {
            translation: eye.translation
                + facing * self.distance
                + Vec3::new(0.0, self.height, 0.0)
                + rotation * Vec3::new(self.offset.x, self.offset.y, 0.0),
            rotation,
            scale: Vec3::ONE,
        }
    }
}
