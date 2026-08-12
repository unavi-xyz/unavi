use wired_prelude::prelude::*;

use crate::wired::scene::types::{
    AlphaMode,
    Material,
};

pub const ACCENT: Color = rgb(0.96, 0.20, 0.16);
pub const SECONDARY: Color = rgb(0.24, 0.72, 1.0);
pub const TERTIARY: Color = rgb(0.42, 0.90, 0.44);
pub const NEUTRAL: Color = rgb(0.90, 0.94, 0.98);
pub const DIM: Color = rgb(0.52, 0.60, 0.70);
pub const SURFACE: Color = rgb(0.14, 0.15, 0.18);

/// Distinct accent colors cycled per tool so each tool's artifact reads
/// differently.
const TOOL_COLORS: [Color; 6] = [
    rgb(0.24, 0.72, 1.00),
    rgb(1.00, 0.62, 0.16),
    rgb(0.42, 0.90, 0.44),
    rgb(0.86, 0.36, 0.98),
    rgb(1.00, 0.32, 0.44),
    rgb(0.30, 0.94, 0.86),
];

#[must_use]
pub const fn tool_color(index: usize) -> Color {
    TOOL_COLORS[index % TOOL_COLORS.len()]
}

pub const GLASS_ALPHA: f32 = 0.15;
pub const GLASS_ALPHA_HOVER: f32 = 0.36;
pub const EMISSIVE_BASE: f32 = 0.12;
pub const EMISSIVE_HOVER: f32 = 0.55;

pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}

pub const fn with_alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a,
    }
}

pub const fn scale(color: Color, f: f32) -> Color {
    Color {
        r: color.r * f,
        g: color.g * f,
        b: color.b * f,
        a: 1.0,
    }
}

/// Translucent frosted panel with a faint self-glow of `color`.
pub const fn glass(color: Color, fill_alpha: f32, emissive: f32) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(AlphaMode::Blend),
        base_color:   Some(with_alpha(color, fill_alpha)),
        double_sided: Some(true),
        emissive:     Some(scale(color, emissive)),
        metallic:     None,
        roughness:    None,
    }
}

/// Bright, emissive edge / glyph material.
pub const fn solid(color: Color, emissive: f32) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(AlphaMode::Blend),
        base_color:   Some(color),
        double_sided: Some(true),
        emissive:     Some(scale(color, emissive)),
        metallic:     None,
        roughness:    None,
    }
}
