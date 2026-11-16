//! One box of a thing, in the thing's own frame.

use wired_guest::{
    math::{
        Color,
        Vec3,
    },
    style::{
        BLUE,
        PANEL,
    },
};

use crate::wired::scene::properties::Material;

const ROUGHNESS: f32 = 0.62;

/// What a part is painted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paint {
    White,
    Blue,
    /// The space's own colour.
    Hue,
}

impl Paint {
    pub fn material(self, hue: Color) -> Material {
        let color = match self {
            Self::White => PANEL,
            Self::Blue => BLUE,
            Self::Hue => hue,
        };
        Material {
            base_color: Some(color),
            metallic: Some(0.0),
            roughness: Some(ROUGHNESS),
            ..Material::default()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub size:   Vec3,
    pub centre: Vec3,
    pub paint:  Paint,
    /// Whether the part carries a collider.
    pub solid:  bool,
}

impl Part {
    pub const fn solid(size: Vec3, centre: Vec3, paint: Paint) -> Self {
        Self {
            size,
            centre,
            paint,
            solid: true,
        }
    }

    pub const fn trim(size: Vec3, centre: Vec3, paint: Paint) -> Self {
        Self {
            size,
            centre,
            paint,
            solid: false,
        }
    }
}
