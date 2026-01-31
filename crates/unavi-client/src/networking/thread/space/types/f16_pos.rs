//! Half-precision f16 delta position type.

use bevy::math::Vec3;
use half::f16;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

/// Delta position with f16 precision (6 bytes).
/// Range: ±65504, Precision: ~0.1% relative.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct F16Pos {
    pub x: f16,
    pub y: f16,
    pub z: f16,
}

impl MaxSize for F16Pos {
    const POSTCARD_MAX_SIZE: usize = 6; // 3× u16
}

impl F16Pos {
    /// Encode delta from baseline position.
    pub fn from_delta(current: Vec3, baseline: Vec3) -> Self {
        let d = current - baseline;
        Self {
            x: f16::from_f32(d.x),
            y: f16::from_f32(d.y),
            z: f16::from_f32(d.z),
        }
    }

    /// Decode delta and apply to baseline position.
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
    fn test_f16_pos_small_delta() {
        let baseline = Vec3::new(10.0, 5.0, 3.0);
        let current = Vec3::new(10.1, 5.2, 3.05);

        let pos = F16Pos::from_delta(current, baseline);
        let restored = pos.apply_to(baseline);

        let error = (current - restored).length();
        assert!(error < 0.001, "small delta error: {error}");
    }

    #[test]
    fn test_f16_pos_medium_delta() {
        let baseline = Vec3::ZERO;
        let current = Vec3::new(1.0, -0.5, 0.25);

        let pos = F16Pos::from_delta(current, baseline);
        let restored = pos.apply_to(baseline);

        let error = (current - restored).length();
        assert!(error < 0.01, "medium delta error: {error}");
    }

    #[test]
    fn test_f16_pos_large_delta() {
        let baseline = Vec3::ZERO;
        let current = Vec3::new(100.0, -50.0, 75.0);

        let pos = F16Pos::from_delta(current, baseline);
        let restored = pos.apply_to(baseline);

        let error = (current - restored).length();
        // ~0.1% relative error at 100m means ~0.1m error
        assert!(error < 0.2, "large delta error: {error}");
    }
}
