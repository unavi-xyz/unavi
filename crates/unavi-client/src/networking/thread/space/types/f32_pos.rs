//! Full-precision f32 position type.

use bevy::math::Vec3;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

/// Absolute position with full f32 precision (12 bytes).
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize)]
pub struct F32Pos {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for F32Pos {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<F32Pos> for Vec3 {
    fn from(p: F32Pos) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_pos_roundtrip() {
        let original = Vec3::new(100.5, -42.25, 0.001);
        let pos: F32Pos = original.into();
        let restored: Vec3 = pos.into();
        assert!((original - restored).length() < 0.0001);
    }
}
