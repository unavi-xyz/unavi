//! What each part of the gate is made of: white stone for the architecture,
//! black glass for the socket's stone, and light.

use wired_guest::math::Color;

use crate::wired::scene::properties::{
    AlphaMode,
    Material,
};

const WHITE: Color = Color {
    r: 0.9,
    g: 0.91,
    b: 0.93,
    a: 1.0,
};
const BLACK: Color = Color {
    r: 0.012,
    g: 0.012,
    b: 0.016,
    a: 1.0,
};
/// What the socket's lights wear with no beacon near.
pub const NEUTRAL: Color = Color {
    r: 0.78,
    g: 0.84,
    b: 0.95,
    a: 1.0,
};

const fn matte(color: Color, roughness: f32, metallic: f32) -> Material {
    Material {
        base_color:   Some(color),
        emissive:     None,
        metallic:     Some(metallic),
        roughness:    Some(roughness),
        alpha_mode:   None,
        alpha_cutoff: None,
        double_sided: None,
    }
}

pub const fn white() -> Material {
    matte(WHITE, 0.62, 0.0)
}

pub const fn black() -> Material {
    matte(BLACK, 0.08, 0.3)
}

/// `color` lit to `level`, from dark at 0 to bright past 1.
pub fn glow(color: Color, level: f32) -> Material {
    let level = level.max(0.0);
    Material {
        base_color: Some(Color {
            r: color.r * 0.2,
            g: color.g * 0.2,
            b: color.b * 0.2,
            a: 1.0,
        }),
        emissive: Some(Color {
            r: color.r * level,
            g: color.g * level,
            b: color.b * level,
            a: 1.0,
        }),
        roughness: Some(0.4),
        ..Material::default()
    }
}

/// Light that adds to what is behind it, `level` from none to full.
pub fn haze(color: Color, level: f32) -> Material {
    let level = level.max(0.0);
    Material {
        base_color: Some(Color {
            a: level.min(1.0),
            ..color
        }),
        emissive: Some(Color {
            r: color.r * level,
            g: color.g * level,
            b: color.b * level,
            a: 1.0,
        }),
        alpha_mode: Some(AlphaMode::Add),
        double_sided: Some(true),
        ..Material::default()
    }
}

/// A see-through beacon in `color`, `opacity` from 0 to 1.
pub fn ghost(color: Color, opacity: f32) -> Material {
    Material {
        base_color: Some(Color {
            a: 0.3 * opacity,
            ..color
        }),
        emissive: Some(Color {
            r: color.r * 0.7 * opacity,
            g: color.g * 0.7 * opacity,
            b: color.b * 0.7 * opacity,
            a: 1.0,
        }),
        alpha_mode: Some(AlphaMode::Blend),
        double_sided: Some(true),
        ..Material::default()
    }
}
