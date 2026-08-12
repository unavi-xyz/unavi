use bevy::{
    prelude::*,
    render::render_resource::{
        AsBindGroup,
        ShaderType,
    },
};
use msdf::atlas::Atlas;

const SHADER: &str = "embedded://bevy_msdf/msdf.wgsl";

/// The baked distance range as a fraction of the field's dimensions, which is
/// the one number tying a shader to the atlas it samples.
#[must_use]
pub fn unit_range(atlas: &Atlas) -> Vec2 {
    Vec2::new(
        atlas.range / atlas.width.max(1) as f32,
        atlas.range / atlas.height.max(1) as f32,
    )
}

#[derive(Clone, Copy, ShaderType, Debug, Default, PartialEq)]
pub struct MsdfSettings {
    pub color:         Vec4,
    pub outline_color: Vec4,
    /// The baked distance range over the field's dimensions; the shader needs
    /// it to convert a distance into screen pixels.
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
    use std::collections::BTreeMap;

    use msdf::atlas::{
        Atlas,
        VerticalMetrics,
    };

    use super::unit_range;

    #[test]
    fn the_unit_range_comes_from_the_field_it_was_baked_against() {
        let unit = unit_range(&Atlas {
            width:    512,
            height:   256,
            range:    8.0,
            vertical: VerticalMetrics::default(),
            glyphs:   BTreeMap::new(),
            kerning:  BTreeMap::new(),
        });
        assert!((unit.x - 8.0 / 512.0).abs() < 1.0e-6);
        assert!((unit.y - 8.0 / 256.0).abs() < 1.0e-6);
    }
}
