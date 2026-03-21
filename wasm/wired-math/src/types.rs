pub use glam::{Vec2, Vec3};

use std::ops::Mul;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    pub const IDENTITY: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        let q = Vec3::new(self.x, self.y, self.z);
        v + 2.0 * self.w * q.cross(v) + 2.0 * q.cross(q.cross(v))
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

    #[must_use]
    pub const fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }
}
