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
use msdf::layout::{
    Align,
    MAX_GLYPHS,
};
use smol_str::SmolStr;

use crate::attributes::{
    AttributeParser,
    ParseError,
};

/// Falls back to body text read at arm's length rather than to zero, so a
/// label whose author omitted a size is legible instead of invisible.
const DEFAULT_SIZE: f32 = 0.02;

/// Em height a label may ask for, in metres. A sign is metres tall; past this
/// the author is not labelling anything.
const MAX_SIZE: f32 = 100.0;

/// Characters one label may carry. Layout refuses more than this, and the
/// payload is whatever a peer wrote, so the text is cut rather than dropped.
const MAX_CHARS: usize = MAX_GLYPHS;

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
/// prim: the payload is stored and re-served untouched, so a document authored
/// against a newer build still draws its text.
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

/// A length or factor a peer wrote, held to a range the renderer can draw. A
/// document may carry any double at all, and a NaN would reach the mesh as a
/// NaN vertex.
fn scalar(value: Option<f64>, fallback: f32, range: std::ops::RangeInclusive<f32>) -> f32 {
    value
        .map(|value| value as f32)
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
        .clamp(*range.start(), *range.end())
}

/// The characters of `value` that fit the cap, cut on a character boundary.
fn truncated(value: &str) -> SmolStr {
    match value.char_indices().nth(MAX_CHARS) {
        Some((at, _)) => SmolStr::new(&value[..at]),
        None => SmolStr::new(value),
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
            width: scalar(attr.outline_width, 0.25, 0.0..=1.0),
        });

        commands.entity(entity).insert((
            MsdfText {
                value:       truncated(&attr.value),
                size:        scalar(attr.size, DEFAULT_SIZE, 0.0..=MAX_SIZE),
                align:       align(attr.align.as_deref()),
                anchor:      anchor(attr.anchor.as_deref()),
                wrap:        attr
                    .wrap
                    .map(|wrap| scalar(Some(wrap), 0.0, 0.0..=MAX_SIZE))
                    .filter(|wrap| *wrap > 0.0),
                line_height: scalar(attr.line_height, 1.0, 0.0..=MAX_SIZE),
                font:        None,
            },
            MsdfStyle {
                color: color(attr.color.as_ref(), Color::WHITE),
                outline,
                emissive: scalar(attr.emissive, 0.0, 0.0..=MAX_SIZE),
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

    #[test]
    fn a_non_finite_length_falls_back_rather_than_reaching_the_mesh() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!((scalar(Some(value), DEFAULT_SIZE, 0.0..=MAX_SIZE) - DEFAULT_SIZE).abs() < 1e-9);
        }
    }

    #[test]
    fn an_absurd_length_is_held_to_the_range() {
        assert!((scalar(Some(1.0e30), DEFAULT_SIZE, 0.0..=MAX_SIZE) - MAX_SIZE).abs() < 1e-6);
        assert!(scalar(Some(-4.0), DEFAULT_SIZE, 0.0..=MAX_SIZE).abs() < 1e-9);
        assert!((scalar(None, DEFAULT_SIZE, 0.0..=MAX_SIZE) - DEFAULT_SIZE).abs() < 1e-9);
    }

    #[test]
    fn a_label_longer_than_the_cap_is_cut_on_a_character_boundary() {
        let value = "漢".repeat(MAX_CHARS + 32);
        let cut = truncated(&value);
        assert_eq!(cut.chars().count(), MAX_CHARS);
        assert_eq!(truncated("hello"), "hello");
    }
}
