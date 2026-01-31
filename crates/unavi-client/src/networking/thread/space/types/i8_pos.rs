//! 1mm resolution i8 delta position type for bones.

use bevy::math::Vec3;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

/// Delta position with 1mm resolution (3 bytes).
/// Range: ±12.7cm from baseline.
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize)]
pub struct I8Pos {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl I8Pos {
    /// Encode delta with 1mm resolution, clamping to range.
    #[expect(clippy::cast_possible_truncation)]
    pub fn from_delta(current: Vec3, baseline: Vec3) -> Self {
        let d = current - baseline;
        Self {
            x: (d.x * 1000.0).clamp(-127.0, 127.0) as i8,
            y: (d.y * 1000.0).clamp(-127.0, 127.0) as i8,
            z: (d.z * 1000.0).clamp(-127.0, 127.0) as i8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode delta and apply to baseline position.
    fn apply_to(pos: I8Pos, baseline: Vec3) -> Vec3 {
        Vec3::new(
            baseline.x + f32::from(pos.x) / 1000.0,
            baseline.y + f32::from(pos.y) / 1000.0,
            baseline.z + f32::from(pos.z) / 1000.0,
        )
    }

    #[test]
    fn test_i8_pos_small_delta() {
        let baseline = Vec3::new(1.0, 2.0, 3.0);
        let current = Vec3::new(1.001, 2.002, 3.003);

        let pos = I8Pos::from_delta(current, baseline);
        let restored = apply_to(pos, baseline);

        let error = (current - restored).length();
        assert!(error < 0.002, "small delta error: {error}");
    }

    #[test]
    fn test_i8_pos_medium_delta() {
        let baseline = Vec3::ZERO;
        let current = Vec3::new(0.05, -0.03, 0.10);

        let pos = I8Pos::from_delta(current, baseline);
        let restored = apply_to(pos, baseline);

        let error = (current - restored).length();
        assert!(error < 0.002, "medium delta error: {error}");
    }

    #[test]
    fn test_i8_pos_clipping() {
        let baseline = Vec3::ZERO;
        // 0.2m = 200mm, exceeds ±127mm range
        let current = Vec3::new(0.2, -0.2, 0.2);

        let pos = I8Pos::from_delta(current, baseline);
        let restored = apply_to(pos, baseline);

        // Should clamp to ±0.127m
        assert!((restored.x - 0.127).abs() < 0.001);
        assert!((restored.y - (-0.127)).abs() < 0.001);
        assert!((restored.z - 0.127).abs() < 0.001);
    }

    #[test]
    fn test_i8_pos_zero_delta() {
        let baseline = Vec3::new(5.0, 10.0, 15.0);
        let current = baseline;

        let pos = I8Pos::from_delta(current, baseline);
        let restored = apply_to(pos, baseline);

        assert!((baseline - restored).length() < 0.001);
    }
}
