use bevy::{
    prelude::*,
    render::render_resource::{
        AsBindGroup,
        ShaderType,
    },
};

const SHADER: &str = "embedded://bevy_msdf/msdf.wgsl";

/// The baked distance range as a fraction of a page's dimensions, which is the
/// one number tying a shader to the atlas it samples.
#[must_use]
pub fn unit_range(range: f32, page_size: u32) -> Vec2 {
    let size = page_size.max(1) as f32;
    Vec2::new(range / size, range / size)
}

#[derive(Clone, Copy, ShaderType, Debug, Default, PartialEq)]
pub struct MsdfSettings {
    pub color:         Vec4,
    pub outline_color: Vec4,
    /// The baked distance range over the page dimensions; the shader needs it
    /// to convert a distance into screen pixels.
    pub unit_range:    Vec2,
    /// How far past the glyph edge the outline reaches, as a fraction of the
    /// distance range. Zero draws none.
    pub outline_width: f32,
    /// Scales the colour past 1.0 so bloom picks the text up; text is drawn
    /// unlit.
    pub emissive:      f32,
}

#[derive(Asset, AsBindGroup, Clone, TypePath, Debug)]
pub struct MsdfMaterial {
    #[uniform(0)]
    pub settings: MsdfSettings,
    #[texture(1)]
    #[sampler(2)]
    pub field:    Handle<Image>,
}

impl Material for MsdfMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    /// Flush text is where z-fighting is worst; the surface loses rather than
    /// flickering.
    fn depth_bias(&self) -> f32 {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::unit_range;

    #[test]
    fn the_unit_range_comes_from_the_page_it_was_baked_against() {
        let unit = unit_range(8.0, 512);
        assert!((unit.x - 8.0 / 512.0).abs() < 1.0e-6);
        assert!((unit.y - 8.0 / 512.0).abs() < 1.0e-6);
    }

    #[test]
    fn a_degenerate_page_never_skips_the_division() {
        let unit = unit_range(4.0, 0);
        assert!(unit.x.is_finite() && unit.y.is_finite());
    }
}
