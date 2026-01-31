//! Player pose types for networking.

use bevy::math::{Quat, Vec3};
use bevy_vrm::BoneName;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use super::{
    pos::{IFramePos, PFramePos},
    quat::PackedQuat,
};

/// I-frame transform: full precision position and rotation.
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize)]
pub struct IFrameTransform {
    pub pos: IFramePos,
    pub rot: PackedQuat,
}

impl IFrameTransform {
    pub fn new(position: Vec3, rotation: Quat) -> Self {
        Self {
            pos: position.into(),
            rot: rotation.into(),
        }
    }
}

/// P-frame transform: delta position and full rotation.
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize)]
pub struct PFrameTransform {
    pub pos: PFramePos,
    pub rot: PackedQuat,
}

impl PFrameTransform {
    pub fn new(current_pos: Vec3, baseline_pos: Vec3, rotation: Quat) -> Self {
        Self {
            pos: PFramePos::from_delta(current_pos, baseline_pos),
            rot: rotation.into(),
        }
    }
}

/// A single bone's transform.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BonePose<T> {
    pub id: BoneName,
    pub transform: T,
}

/// Complete player pose with root and optional bones.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerPose<T> {
    pub root: T,
    pub bones: Vec<BonePose<T>>,
}

pub type PlayerIFrame = PlayerPose<IFrameTransform>;
pub type PlayerPFrame = PlayerPose<PFrameTransform>;
