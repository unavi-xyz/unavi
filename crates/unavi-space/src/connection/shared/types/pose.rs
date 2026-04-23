use bevy::platform::collections::HashMap;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};
use serde_vrm::vrm0::BoneName;

use super::{
    IFrame, PFrame, f16_vec3::F16Vec3, f32_vec3::F32Vec3, i8_vec3::I8Vec3, quat::PackedQuat,
};

#[derive(Clone, Debug,MaxSize, Serialize, Deserialize, Default, )]
pub struct Transform<T> {
    pub tra: T,
    pub rot: PackedQuat,
}

trait PosePrecision {
    type RootTransform: MaxSize + Serialize + for<'a> Deserialize<'a> + Default;
    type BoneTransform: MaxSize + Serialize + for<'a> Deserialize<'a> + Default;
}

impl PosePrecision for IFrame {
    type RootTransform = Transform<F32Vec3>;
    type BoneTransform = Transform<F16Vec3>;
}

impl PosePrecision for PFrame {
    type RootTransform = Transform<F16Vec3>;
    type BoneTransform = Transform<I8Vec3>;
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Pose<T>
where
    T: PosePrecision + Serialize + for<'a> Deserialize<'a>,
{
    pub root: T::RootTransform,
    pub bones: HashMap<BoneName, T::BoneTransform>,
}

pub const MAX_POSE_BONES: usize = 12;

impl<T> MaxSize for Pose<T>
where
    T: PosePrecision + Serialize + for<'a> Deserialize<'a>,
{
    const POSTCARD_MAX_SIZE: usize = T::RootTransform::POSTCARD_MAX_SIZE
        // HashMap varint prefix
        + 1 
        // N * (Key + Value)
        + MAX_POSE_BONES * (1 + T::BoneTransform::POSTCARD_MAX_SIZE);
}
