//! Agent pose types for networking.

use bevy::math::{Quat, Vec3};
use bevy_vrm::BoneName;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use super::{F16Pos, F32Pos, I8Pos, PackedQuat};

/// I-frame transform: full precision position and rotation.
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize)]
pub struct IFrameTransform {
    pub pos: F32Pos,
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

/// P-frame root transform: f16 delta from I-frame baseline.
/// Uses half-precision for ~0.1% relative error, ±65504 range.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PFrameRootTransform {
    pub pos: F16Pos,
    pub rot: PackedQuat,
}

impl MaxSize for PFrameRootTransform {
    const POSTCARD_MAX_SIZE: usize = F16Pos::POSTCARD_MAX_SIZE + PackedQuat::POSTCARD_MAX_SIZE;
}

impl PFrameRootTransform {
    /// Create from current position and I-frame baseline.
    pub fn new(current_pos: Vec3, baseline_pos: Vec3, rotation: Quat) -> Self {
        Self {
            pos: F16Pos::from_delta(current_pos, baseline_pos),
            rot: rotation.into(),
        }
    }
}

/// P-frame bone transform: i8 delta from I-frame baseline.
/// Uses 1mm resolution, ±12.7cm range.
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize)]
pub struct PFrameTransform {
    pub pos: I8Pos,
    pub rot: PackedQuat,
}

impl PFrameTransform {
    /// Create from current position and I-frame baseline.
    pub fn new(current_pos: Vec3, baseline_pos: Vec3, rotation: Quat) -> Self {
        Self {
            pos: I8Pos::from_delta(current_pos, baseline_pos),
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

/// Complete agent pose with root and optional bones.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentPose<T> {
    pub root: T,
    pub bones: Vec<BonePose<T>>,
}

pub type AgentIFrame = AgentPose<IFrameTransform>;

/// P-frame agent pose with delta root and delta bones.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentPFrame {
    pub root: PFrameRootTransform,
    pub bones: Vec<BonePose<PFrameTransform>>,
}
