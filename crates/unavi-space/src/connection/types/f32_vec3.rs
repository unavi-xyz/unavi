use bevy::math::Vec3;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize, Default)]
pub struct F32Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for F32Vec3 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<F32Vec3> for Vec3 {
    fn from(p: F32Vec3) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_pos_roundtrip() {
        let original = Vec3::new(100.5, -42.25, 0.001);
        let pos: F32Vec3 = original.into();
        let restored: Vec3 = pos.into();
        assert!((original - restored).length() < 0.0001);
    }
}
