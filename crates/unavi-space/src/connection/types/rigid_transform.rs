use bevy::transform::components::Transform;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::connection::types::{
    f16_vec3::F16Vec3, f32_vec3::F32Vec3, i8_vec3::I8Vec3, quat::PackedQuat,
};

#[derive(Clone, Debug, MaxSize, Serialize, Deserialize, Default)]
pub struct RigidTransform<T> {
    pub tra: T,
    pub rot: PackedQuat,
}

impl RigidTransform<F16Vec3> {
    pub fn delta(current: &RigidTransform<F32Vec3>, baseline: &RigidTransform<F32Vec3>) -> Self {
        Self {
            tra: F16Vec3::from_delta(current.tra, baseline.tra),
            rot: current.rot,
        }
    }
}

impl RigidTransform<I8Vec3> {
    pub fn delta(current: &RigidTransform<F16Vec3>, baseline: &RigidTransform<F16Vec3>) -> Self {
        Self {
            tra: I8Vec3::from_delta(current.tra, baseline.tra),
            rot: current.rot,
        }
    }
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
        Self {
            translation: val.tra.into(),
            rotation: val.rot.into(),
            ..Default::default()
        }
    }
}
