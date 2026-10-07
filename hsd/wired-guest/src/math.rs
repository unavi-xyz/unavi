//! Rust types for `wired:core/math`, mapped in by every guest's
//! `generate!` call.
//!
//! `Vec2`/`Vec3` are `glam`'s own types: their layout is a plain `{x, y(, z)}`
//! struct, so wit-bindgen's record lowering works unmodified and a guest gets
//! `glam`'s full vector API for free. `glam::Quat` has no such layout (it is
//! SIMD-backed and its fields are private), so `Quat` here is a plain
//! `{x, y, z, w}` struct that converts to and from `glam::Quat` for actual
//! rotation math; `Transform` and `Color` have no `glam` equivalent at all
//! and are plain structs for the same reason.

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

/// A unit quaternion. `glam::Quat` is SIMD-backed with private fields, so
/// this is the wire layout; convert with [`Quat::from`]/[`glam::Quat::from`]
/// to do rotation math.
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

    /// A rotation of `radians` about the Y axis.
    #[must_use]
    pub fn from_rotation_y(radians: f32) -> Self {
        Self::from(glam::Quat::from_rotation_y(radians))
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
        glam::Quat::from(self) * v
    }
}

impl Mul for Quat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::from(glam::Quat::from(self) * glam::Quat::from(rhs))
    }
}

/// Scale, then rotation, then translation, matching
/// `wired:core/math.transform`.
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

    /// `translation`, with identity rotation and unit scale.
    #[must_use]
    pub const fn from_translation(translation: Vec3) -> Self {
        Self::new(translation, Quat::IDENTITY, Vec3::ONE)
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

/// Linear RGB with straight alpha, matching `wired:core/math.color`.
#[derive(Debug, Default, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[expect(clippy::many_single_char_names)]
    #[must_use]
    pub fn hsv(h: f32, s: f32, v: f32) -> Self {
        let i = (h * 6.0).floor() as u32 % 6;
        let f = h.mul_add(6.0, -(h * 6.0).floor());
        let p = v * (1.0 - s);
        let q = v * f.mul_add(-s, 1.0);
        let t = v * (1.0 - f).mul_add(-s, 1.0);
        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        Self::rgb(r, g, b)
    }
}

/// A half-line in world space, matching `wired:core/math.ray`.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Ray {
    pub origin:    Vec3,
    pub direction: Vec3,
}
