use bevy::math::Vec3;
use postcard::experimental::max_size::MaxSize;
use serde::{
    Deserialize,
    Serialize,
};

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

    /// No quantization here, so a component mismatch (e.g. `y`/`z` swapped in
    /// a refactor) is the only way this roundtrip can fail; exact equality
    /// catches that.
    #[test]
    fn roundtrip_is_exact() {
        let original = Vec3::new(100.5, -42.25, 0.001);
        let pos: F32Vec3 = original.into();
        let restored: Vec3 = pos.into();
        assert_eq!(original, restored);
    }
}
