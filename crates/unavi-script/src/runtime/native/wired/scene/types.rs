use bevy::prelude::*;

use super::bindings::wired::math::types::{
    Quat as WitQuat, Transform as WitTransform, Vec3 as WitVec3,
};

impl From<bevy::math::Vec3> for WitVec3 {
    fn from(v: bevy::math::Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<WitVec3> for bevy::math::Vec3 {
    fn from(v: WitVec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl From<bevy::math::Quat> for WitQuat {
    fn from(q: bevy::math::Quat) -> Self {
        Self {
            x: q.x,
            y: q.y,
            z: q.z,
            w: q.w,
        }
    }
}

impl From<WitQuat> for bevy::math::Quat {
    fn from(q: WitQuat) -> Self {
        Self::from_xyzw(q.x, q.y, q.z, q.w)
    }
}

impl From<Transform> for WitTransform {
    fn from(t: Transform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl From<WitTransform> for Transform {
    fn from(t: WitTransform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl From<GlobalTransform> for WitTransform {
    fn from(gt: GlobalTransform) -> Self {
        let (scale, rotation, translation) = gt.to_scale_rotation_translation();
        Self {
            translation: translation.into(),
            rotation: rotation.into(),
            scale: scale.into(),
        }
    }
}
