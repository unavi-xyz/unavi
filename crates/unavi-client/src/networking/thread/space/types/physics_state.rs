//! Physics state types for dynamic object serialization.

use bevy::math::{Quat, Vec3};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use super::{
    f16_pos::F16Pos,
    f32_pos::F32Pos,
    quat::PackedQuat,
    velocity::{F16Vel, F32Vel},
};

/// Full physics state for I-frames.
///
/// Contains absolute values for position, rotation, velocity, and angular velocity.
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize)]
pub struct PhysicsIFrame {
    pub pos: F32Pos,
    pub rot: PackedQuat,
    pub vel: F32Vel,
    pub ang_vel: F32Vel,
}

impl PhysicsIFrame {
    pub fn new(pos: Vec3, rot: Quat, vel: Vec3, ang_vel: Vec3) -> Self {
        Self {
            pos: pos.into(),
            rot: rot.into(),
            vel: vel.into(),
            ang_vel: ang_vel.into(),
        }
    }
}

/// Delta physics state for P-frames.
///
/// Contains deltas relative to the I-frame baseline for position and velocities.
/// Rotation is absolute (packed quaternion is already compact).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PhysicsPFrame {
    pub pos: F16Pos,
    pub rot: PackedQuat,
    pub vel: F16Vel,
    pub ang_vel: F16Vel,
}

impl MaxSize for PhysicsPFrame {
    const POSTCARD_MAX_SIZE: usize = F16Pos::POSTCARD_MAX_SIZE
        + PackedQuat::POSTCARD_MAX_SIZE
        + F16Vel::POSTCARD_MAX_SIZE
        + F16Vel::POSTCARD_MAX_SIZE;
}

impl PhysicsPFrame {
    /// Create from current state and I-frame baseline.
    pub fn new(
        current_pos: Vec3,
        baseline_pos: Vec3,
        rot: Quat,
        current_vel: Vec3,
        baseline_vel: Vec3,
        current_ang_vel: Vec3,
        baseline_ang_vel: Vec3,
    ) -> Self {
        Self {
            pos: F16Pos::from_delta(current_pos, baseline_pos),
            rot: rot.into(),
            vel: F16Vel::from_delta(current_vel, baseline_vel),
            ang_vel: F16Vel::from_delta(current_ang_vel, baseline_ang_vel),
        }
    }
}

/// Baseline state for computing P-frame deltas.
#[derive(Clone, Copy, Debug, Default)]
pub struct PhysicsBaseline {
    pub pos: Vec3,
    pub vel: Vec3,
    pub ang_vel: Vec3,
}

impl From<&PhysicsIFrame> for PhysicsBaseline {
    fn from(iframe: &PhysicsIFrame) -> Self {
        Self {
            pos: iframe.pos.into(),
            vel: iframe.vel.into(),
            ang_vel: iframe.ang_vel.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_iframe() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
        let vel = Vec3::new(0.5, 0.0, -0.5);
        let ang_vel = Vec3::new(0.1, 0.2, 0.0);

        let iframe = PhysicsIFrame::new(pos, rot, vel, ang_vel);

        let pos_back: Vec3 = iframe.pos.into();
        assert!((pos_back - pos).length() < 0.001);

        let vel_back: Vec3 = iframe.vel.into();
        assert_eq!(vel_back, vel);
    }

    #[test]
    fn test_physics_pframe_delta() {
        let baseline_pos = Vec3::new(1.0, 2.0, 3.0);
        let baseline_vel = Vec3::new(0.5, 0.0, -0.5);
        let baseline_ang_vel = Vec3::new(0.1, 0.2, 0.0);

        let current_pos = Vec3::new(1.1, 2.1, 3.1);
        let current_vel = Vec3::new(0.6, 0.1, -0.4);
        let current_ang_vel = Vec3::new(0.15, 0.25, 0.05);
        let rot = Quat::IDENTITY;

        let pframe = PhysicsPFrame::new(
            current_pos,
            baseline_pos,
            rot,
            current_vel,
            baseline_vel,
            current_ang_vel,
            baseline_ang_vel,
        );

        // Reconstruct and verify.
        let reconstructed_pos = pframe.pos.apply_to(baseline_pos);
        let reconstructed_vel = pframe.vel.apply_to(baseline_vel);
        let reconstructed_ang_vel = pframe.ang_vel.apply_to(baseline_ang_vel);

        // Allow f16 precision error.
        assert!((reconstructed_pos - current_pos).length() < 0.01);
        assert!((reconstructed_vel - current_vel).length() < 0.01);
        assert!((reconstructed_ang_vel - current_ang_vel).length() < 0.01);
    }
}
