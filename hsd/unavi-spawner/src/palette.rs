//! Colour defaults for the spawner's cube and its preview.

use wired_guest::math::Color;

use crate::wired::scene::properties::{
    AlphaMode,
    Material,
};

pub const DEFAULT: Color = Color {
    r: 0.24,
    g: 0.72,
    b: 1.0,
    a: 1.0,
};

const fn scale(color: Color, f: f32) -> Color {
    Color {
        r: color.r * f,
        g: color.g * f,
        b: color.b * f,
        a: 1.0,
    }
}

/// Opaque physical cube material with a faint self-glow.
pub const fn cube(color: Color) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(color),
        double_sided: None,
        emissive:     Some(scale(color, 0.15)),
        metallic:     Some(0.2),
        roughness:    Some(0.6),
    }
}

/// Translucent preview material shown above the artifact.
pub const fn preview(color: Color) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(AlphaMode::Blend),
        base_color:   Some(Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 0.4,
        }),
        double_sided: Some(true),
        emissive:     Some(scale(color, 0.5)),
        metallic:     None,
        roughness:    None,
    }
}
