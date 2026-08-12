use bevy::{
    light::NotShadowCaster,
    prelude::*,
};
use msdf::{
    atlas::Atlas,
    layout::{
        Align,
        LayoutOpts,
        layout,
    },
};
use smol_str::SmolStr;

use crate::{
    font::{
        DefaultFont,
        MsdfFont,
    },
    material::{
        MsdfMaterial,
        MsdfSettings,
        unit_range,
    },
    mesh::{
        Anchor,
        build,
    },
};

/// A string drawn in the world. Split from [`MsdfStyle`] so a style change
/// never re-tessellates the mesh.
#[derive(Component, Debug, Clone)]
#[require(Transform, Visibility, MsdfStyle)]
pub struct MsdfText {
    pub value:       SmolStr,
    /// Em height, in metres. 0.02 is body text read at arm's length.
    pub size:        f32,
    pub align:       Align,
    pub anchor:      Anchor,
    /// Wrap width in metres. `None` breaks only on newlines.
    pub wrap:        Option<f32>,
    pub line_height: f32,
    /// `None` draws with [`DefaultFont`].
    pub font:        Option<Handle<MsdfFont>>,
}

impl Default for MsdfText {
    fn default() -> Self {
        Self {
            value:       SmolStr::default(),
            size:        0.02,
            align:       Align::Left,
            anchor:      Anchor::Baseline,
            wrap:        None,
            line_height: 1.0,
            font:        None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    pub color: Color,
    /// Fraction of the baked distance range the outline reaches out to.
    /// Beyond roughly 0.4 the field runs out of gradient and the edge breaks
    /// up.
    pub width: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct MsdfStyle {
    pub color:    Color,
    /// Keeps text legible over backgrounds the text did not choose.
    pub outline:  Option<Outline>,
    pub emissive: f32,
}

impl Default for MsdfStyle {
    fn default() -> Self {
        Self {
            color:    Color::WHITE,
            outline:  None,
            emissive: 0.0,
        }
    }
}

/// Characters the font had no glyph for, per entity; [`report_missing_glyphs`]
/// logs each new one.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MissingGlyphs(pub usize);

fn settings(style: &MsdfStyle, atlas: &Atlas) -> MsdfSettings {
    let outline = style.outline.unwrap_or(Outline {
        color: Color::NONE,
        width: 0.0,
    });
    MsdfSettings {
        color:         LinearRgba::from(style.color).to_vec4(),
        outline_color: LinearRgba::from(outline.color).to_vec4(),
        unit_range:    unit_range(atlas),
        outline_width: outline.width.max(0.0),
        emissive:      style.emissive.max(0.0),
    }
}

fn font<'a>(
    text: &MsdfText,
    fonts: &'a Assets<MsdfFont>,
    default: &DefaultFont,
) -> Option<&'a MsdfFont> {
    fonts.get(text.font.as_ref().unwrap_or(&default.0))
}

pub fn rebuild_text(
    changed: Query<(Entity, &MsdfText, &MsdfStyle), Or<(Changed<MsdfText>, Without<Mesh3d>)>>,
    fonts: Res<Assets<MsdfFont>>,
    default: Res<DefaultFont>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MsdfMaterial>>,
    mut commands: Commands,
) {
    for (entity, text, style) in &changed {
        let Some(font) = font(text, &fonts, &default) else {
            continue;
        };
        let laid = match layout(
            &text.value,
            &font.atlas,
            &LayoutOpts {
                size: text.size,
                wrap: text.wrap,
                align: text.align,
                line_height: text.line_height,
                ..Default::default()
            },
        ) {
            Ok(laid) => laid,
            Err(err) => {
                error!("{entity}: {err}");
                continue;
            }
        };

        commands.entity(entity).insert((
            Mesh3d(meshes.add(build(&laid, text.anchor))),
            MeshMaterial3d(materials.add(MsdfMaterial {
                settings: settings(style, &font.atlas),
                field:    font.field.clone(),
            })),
            MissingGlyphs(laid.missing),
            // A glyph quad casting a shadow is a rectangle of shade with no
            // letter in it.
            NotShadowCaster,
        ));
    }
}

/// Restyles without rebuilding the mesh, so a colour fading every frame is
/// cheap.
pub fn restyle_text(
    changed: Query<
        (&MsdfText, &MsdfStyle, &MeshMaterial3d<MsdfMaterial>),
        Or<(Changed<MsdfStyle>, Changed<MeshMaterial3d<MsdfMaterial>>)>,
    >,
    fonts: Res<Assets<MsdfFont>>,
    default: Res<DefaultFont>,
    mut materials: ResMut<Assets<MsdfMaterial>>,
) {
    for (text, style, handle) in &changed {
        let (Some(font), Some(mut material)) =
            (font(text, &fonts, &default), materials.get_mut(handle))
        else {
            continue;
        };
        material.settings = settings(style, &font.atlas);
    }
}

pub fn report_missing_glyphs(
    changed: Query<(Entity, &MsdfText, &MissingGlyphs), Changed<MissingGlyphs>>,
) {
    for (entity, text, missing) in &changed {
        if missing.0 > 0 {
            error!(
                "{entity}: {} of {:?} have no glyph in this font and were dropped",
                missing.0, text.value,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atlas() -> Atlas {
        crate::font::load(
            include_bytes!(concat!(env!("OUT_DIR"), "/latin.png")),
            include_bytes!(concat!(env!("OUT_DIR"), "/latin.bin")),
        )
        .expect("shipped font")
        .0
    }

    #[test]
    fn an_unstyled_outline_contributes_nothing() {
        let settings = settings(&MsdfStyle::default(), &atlas());
        assert!(settings.outline_width.abs() < f32::EPSILON);
        assert!(settings.outline_color.w.abs() < f32::EPSILON);
    }

    #[test]
    fn a_negative_outline_never_reaches_the_shader() {
        let settings = settings(
            &MsdfStyle {
                outline: Some(Outline {
                    color: Color::BLACK,
                    width: -1.0,
                }),
                emissive: -3.0,
                ..Default::default()
            },
            &atlas(),
        );
        assert!(settings.outline_width >= 0.0);
        assert!(settings.emissive >= 0.0);
    }
}
