use bevy::transform::components::Transform;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::connection::types::{f32_vec3::F32Vec3, quat::PackedQuat};

#[derive(Clone, Debug, MaxSize, Serialize, Deserialize, Default)]
pub struct RigidTransform<T> {
    pub tra: T,
    pub rot: PackedQuat,
}

impl From<&Transform> for RigidTransform<F32Vec3> {
    fn from(value: &Transform) -> Self {
        Self {
            tra: value.translation.into(),
            rot: value.rotation.into(),
        }
    }
}

impl From<RigidTransform<F32Vec3>> for Transform {
    fn from(val: RigidTransform<F32Vec3>) -> Self {
        Transform {
            translation: val.tra.into(),
            rotation: val.rot.into(),
            ..Default::default()
        }
    }
}
