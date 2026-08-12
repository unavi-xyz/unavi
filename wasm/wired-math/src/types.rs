use std::ops::{
    Mul,
    Neg,
};

pub use glam::{
    Vec2,
    Vec3,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
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

    #[must_use]
    pub const fn conjugate(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    #[must_use]
    pub const fn inverse(self) -> Self {
        self.conjugate()
    }

    #[must_use]
    pub fn normalize(self) -> Self {
        Self::from(glam::Quat::from(self).normalize())
    }

    /// Decomposes into a rotation axis and angle in radians.
    #[must_use]
    pub fn to_axis_angle(self) -> (Vec3, f32) {
        glam::Quat::from(self).to_axis_angle()
    }
}

impl From<Quat> for glam::Quat {
    fn from(q: Quat) -> Self {
        Self::from_xyzw(q.x, q.y, q.z, q.w)
    }
}

impl From<glam::Quat> for Quat {
    fn from(q: glam::Quat) -> Self {
        Self::new(q.x, q.y, q.z, q.w)
    }
}

impl Neg for Quat {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        let q = Vec3::new(self.x, self.y, self.z);
        v + 2.0 * self.w * q.cross(v) + 2.0 * q.cross(q.cross(v))
    }
}

impl Mul for Quat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::from(glam::Quat::from(self) * glam::Quat::from(rhs))
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation:    Quat,
    pub scale:       Vec3,
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

    #[must_use]
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    #[must_use]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.translation + self.rotation * (self.scale * point)
    }
}
