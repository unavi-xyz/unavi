use bevy::prelude::*;
use bevy_msdf::{
    billboard::Billboard,
    mesh::Anchor,
    text::{
        MsdfStyle,
        MsdfText,
        Outline,
    },
};
use hsd::attributes::{
    Attribute,
    material::ColorVec,
    text::TextAttr,
};
use msdf::layout::Align;
use smol_str::SmolStr;

use crate::attributes::{
    AttributeParser,
    ParseError,
};

/// Falls back to body text read at arm's length rather than to zero, so a
/// label whose author omitted a size is legible instead of invisible.
const DEFAULT_SIZE: f32 = 0.02;

#[derive(Component, Debug, Clone)]
pub struct TextData(pub TextAttr);

pub struct TextParser;

impl AttributeParser for TextParser {
    fn key(&self) -> &'static str {
        TextAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError> {
        match payload {
            Some(payload) => {
                commands
                    .entity(prim)
                    .insert(TextData(TextAttr::decode(payload)?));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<(TextData, MsdfText, MsdfStyle, Billboard, Mesh3d)>();
            }
        }
        Ok(())
    }
}

/// A short vector pads with opaque white rather than refusing the colour: the
/// payload is whatever a peer wrote, and a malformed one costs a wrong shade,
/// not a missing label.
fn color(value: Option<&ColorVec>, fallback: Color) -> Color {
    value.map_or(fallback, |value| {
        let channel = |index: usize| value.0.get(index).copied().unwrap_or(1.0) as f32;
        Color::linear_rgba(channel(0), channel(1), channel(2), channel(3))
    })
}

/// An unrecognized variant falls back to the default rather than refusing the
/// prim: the payload is stored and re-served untouched either way, so a
/// document authored against a newer build still draws its text.
fn align(value: Option<&str>) -> Align {
    match value {
        Some("center") => Align::Center,
        Some("right") => Align::Right,
        _ => Align::Left,
    }
}

fn anchor(value: Option<&str>) -> Anchor {
    match value {
        Some("top") => Anchor::Top,
        Some("middle") => Anchor::Middle,
        Some("bottom") => Anchor::Bottom,
        _ => Anchor::Baseline,
    }
}

fn billboard(value: Option<&str>) -> Option<Billboard> {
    match value {
        Some("yaw") => Some(Billboard::Yaw),
        Some("full") => Some(Billboard::Full),
        _ => None,
    }
}

pub fn apply_text(changed: Query<(Entity, &TextData), Changed<TextData>>, mut commands: Commands) {
    for (entity, data) in &changed {
        let attr = &data.0;

        let outline = attr.outline.as_ref().map(|value| Outline {
            color: color(Some(value), Color::BLACK),
            width: attr.outline_width.unwrap_or(0.25) as f32,
        });

        commands.entity(entity).insert((
            MsdfText {
                value:       SmolStr::new(&attr.value),
                size:        attr.size.map_or(DEFAULT_SIZE, |size| size as f32),
                align:       align(attr.align.as_deref()),
                anchor:      anchor(attr.anchor.as_deref()),
                wrap:        attr.wrap.map(|wrap| wrap as f32),
                line_height: attr.line_height.unwrap_or(1.0) as f32,
                font:        None,
            },
            MsdfStyle {
                color: color(attr.color.as_ref(), Color::WHITE),
                outline,
                emissive: attr.emissive.unwrap_or(0.0) as f32,
            },
        ));

        match billboard(attr.billboard.as_deref()) {
            Some(billboard) => {
                commands.entity(entity).insert(billboard);
            }
            None => {
                commands.entity(entity).remove::<Billboard>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_variant_falls_back_rather_than_refusing() {
        assert_eq!(align(Some("justify")), Align::Left);
        assert_eq!(anchor(Some("hanging")), Anchor::Baseline);
        assert_eq!(billboard(Some("spherical")), None);
    }

    #[test]
    fn every_variant_the_format_names_is_understood() {
        assert_eq!(align(Some("center")), Align::Center);
        assert_eq!(align(Some("right")), Align::Right);
        assert_eq!(anchor(Some("middle")), Anchor::Middle);
        assert_eq!(anchor(Some("top")), Anchor::Top);
        assert_eq!(anchor(Some("bottom")), Anchor::Bottom);
        assert_eq!(billboard(Some("yaw")), Some(Billboard::Yaw));
        assert_eq!(billboard(Some("full")), Some(Billboard::Full));
    }

    #[test]
    fn a_short_colour_vector_pads_rather_than_panicking() {
        let padded = color(Some(&ColorVec(vec![0.5])), Color::WHITE);
        assert_eq!(padded, Color::linear_rgba(0.5, 1.0, 1.0, 1.0));
    }
}
