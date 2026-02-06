//! Velocity types for physics state serialization.

use bevy::math::Vec3;
use half::f16;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

/// Full precision velocity (m/s).
#[derive(Clone, Copy, Debug, Default, MaxSize, Serialize, Deserialize)]
pub struct F32Vel {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for F32Vel {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<F32Vel> for Vec3 {
    fn from(v: F32Vel) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

/// Half precision velocity delta (m/s).
///
/// Uses f16 for each component, providing ~0.1% relative error.
/// Range: ±65504 m/s (more than sufficient for physics).
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct F16Vel {
    pub x: f16,
    pub y: f16,
    pub z: f16,
}

impl MaxSize for F16Vel {
    const POSTCARD_MAX_SIZE: usize = 6; // 3 * 2 bytes
}

impl F16Vel {
    /// Create from current velocity and baseline velocity.
    pub fn from_delta(current: Vec3, baseline: Vec3) -> Self {
        let delta = current - baseline;
        Self {
            x: f16::from_f32(delta.x),
            y: f16::from_f32(delta.y),
            z: f16::from_f32(delta.z),
        }
    }

    /// Apply delta to baseline velocity to reconstruct current velocity.
    pub fn apply_to(self, baseline: Vec3) -> Vec3 {
        Vec3::new(
            baseline.x + self.x.to_f32(),
            baseline.y + self.y.to_f32(),
            baseline.z + self.z.to_f32(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_vel_roundtrip() {
        let v = Vec3::new(1.5, -2.3, 4.7);
        let vel: F32Vel = v.into();
        let back: Vec3 = vel.into();
        assert_eq!(v, back);
    }

    #[test]
    fn test_f16_vel_delta() {
        let baseline = Vec3::new(1.0, 2.0, 3.0);
        let current = Vec3::new(1.5, 2.5, 3.5);
        let delta = F16Vel::from_delta(current, baseline);
        let reconstructed = delta.apply_to(baseline);

        // Allow small error due to f16 precision.
        assert!((reconstructed.x - current.x).abs() < 0.01);
        assert!((reconstructed.y - current.y).abs() < 0.01);
        assert!((reconstructed.z - current.z).abs() < 0.01);
    }
}
