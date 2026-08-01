use wired_prelude::prelude::*;

use crate::wired::scene::types::{
    AlphaMode,
    Material,
};

pub const DEFAULT: Color = Color {
    r: 1.0,
    g: 0.62,
    b: 0.16,
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

/// Bright additive beam material for the physics tractor ray.
pub const fn beam(color: Color) -> Material {
    Material {
        alpha_cutoff:               None,
        alpha_mode:                 Some(AlphaMode::Add),
        base_color:                 Some(color),
        base_color_texture:         None,
        double_sided:               Some(true),
        emissive:                   Some(scale(color, 2.0)),
        emissive_texture:           None,
        metallic:                   None,
        metallic_roughness_texture: None,
        normal_texture:             None,
        occlusion_texture:          None,
        roughness:                  None,
    }
}
