//! An avatar pose: root transform plus tracked bones, at keyframe or delta
//! precision.

use bevy::platform::collections::HashMap;
use postcard::experimental::max_size::MaxSize;
use serde::{
    Deserialize,
    Serialize,
};
use serde_vrm::vrm0::BoneName;

use super::{
    Delta,
    Keyframe,
    f16_vec3::F16Vec3,
    f32_vec3::F32Vec3,
    i8_vec3::I8Vec3,
};
use crate::link::codec::rigid_transform::RigidTransform;

pub trait PosePrecision {
    type RootTransform: MaxSize + Serialize + for<'a> Deserialize<'a> + Default + Clone;
    type BoneTransform: MaxSize + Serialize + for<'a> Deserialize<'a> + Default + Clone;
}

impl PosePrecision for Keyframe {
    type RootTransform = RigidTransform<F32Vec3>;
    type BoneTransform = RigidTransform<F16Vec3>;
}

impl PosePrecision for Delta {
    type RootTransform = RigidTransform<F16Vec3>;
    type BoneTransform = RigidTransform<I8Vec3>;
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Pose<T>
where
    T: PosePrecision + Serialize + for<'a> Deserialize<'a> + Clone,
{
    pub root:  T::RootTransform,
    pub bones: HashMap<BoneName, T::BoneTransform>,
}

pub const MAX_POSE_BONES: usize = 12;

impl<T> MaxSize for Pose<T>
where
    T: PosePrecision + Serialize + for<'a> Deserialize<'a> + Clone,
{
    // Root
    // + HashMap varint prefix (1 byte)
    // + N * [Key (1 byte)  + Value]
    const POSTCARD_MAX_SIZE: usize = T::RootTransform::POSTCARD_MAX_SIZE
        + 1
        + MAX_POSE_BONES * (1 + T::BoneTransform::POSTCARD_MAX_SIZE);
}
