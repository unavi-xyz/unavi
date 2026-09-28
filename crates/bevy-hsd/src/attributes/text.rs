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
use hsd::{
    attributes::text::{
        TextAlign,
        TextAnchor,
        TextAttr,
        TextBillboard,
    },
    property::Payload,
};
use msdf::layout::{
    Align,
    MAX_GLYPHS,
};
use smol_str::SmolStr;

use crate::attributes::{
    ParseError,
    color::from_color_vec,
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

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    match payload {
        Some(payload) => build(commands, prim, &TextAttr::decode(payload)?),
        None => {
            commands
                .entity(prim)
                .remove::<(MsdfText, MsdfStyle, Billboard, Mesh3d)>();
        }
    }
    Ok(())
}

fn build(commands: &mut Commands, entity: Entity, attr: &TextAttr) {
    let outline = attr.outline.as_ref().map(|value| Outline {
        color: from_color_vec(Some(value), Color::BLACK),
        width: scalar(attr.outline_width, 0.25, 0.0..=1.0),
    });

    commands.entity(entity).insert((
        MsdfText {
            value:       truncated(&attr.value),
            size:        scalar(attr.size, DEFAULT_SIZE, 0.0..=MAX_SIZE),
            align:       align(attr.align),
            anchor:      anchor(attr.anchor),
            wrap:        attr
                .wrap
                .map(|wrap| scalar(Some(wrap), 0.0, 0.0..=MAX_SIZE))
                .filter(|wrap| *wrap > 0.0),
            line_height: scalar(attr.line_height, 1.0, 0.0..=MAX_SIZE),
            font:        None,
        },
        MsdfStyle {
            color: from_color_vec(attr.color.as_ref(), Color::WHITE),
            outline,
            emissive: scalar(attr.emissive, 0.0, 0.0..=MAX_SIZE),
        },
    ));

    match billboard(attr.billboard) {
        Some(billboard) => {
            commands.entity(entity).insert(billboard);
        }
        None => {
            commands.entity(entity).remove::<Billboard>();
        }
    }
}

const fn align(value: Option<TextAlign>) -> Align {
    match value {
        Some(TextAlign::Center) => Align::Center,
        Some(TextAlign::Right) => Align::Right,
        Some(TextAlign::Left) | None => Align::Left,
    }
}

const fn anchor(value: Option<TextAnchor>) -> Anchor {
    match value {
        Some(TextAnchor::Top) => Anchor::Top,
        Some(TextAnchor::Middle) => Anchor::Middle,
        Some(TextAnchor::Bottom) => Anchor::Bottom,
        Some(TextAnchor::Baseline) | None => Anchor::Baseline,
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

const fn billboard(value: Option<TextBillboard>) -> Option<Billboard> {
    match value {
        Some(TextBillboard::Yaw) => Some(Billboard::Yaw),
        Some(TextBillboard::Full) => Some(Billboard::Full),
        Some(TextBillboard::None) | None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_the_format_names_is_understood() {
        assert_eq!(align(None), Align::Left);
        assert_eq!(align(Some(TextAlign::Center)), Align::Center);
        assert_eq!(align(Some(TextAlign::Right)), Align::Right);
        assert_eq!(anchor(None), Anchor::Baseline);
        assert_eq!(anchor(Some(TextAnchor::Middle)), Anchor::Middle);
        assert_eq!(anchor(Some(TextAnchor::Top)), Anchor::Top);
        assert_eq!(anchor(Some(TextAnchor::Bottom)), Anchor::Bottom);
        assert_eq!(billboard(None), None);
        assert_eq!(billboard(Some(TextBillboard::Yaw)), Some(Billboard::Yaw));
        assert_eq!(billboard(Some(TextBillboard::Full)), Some(Billboard::Full));
    }

    #[test]
    fn a_non_finite_length_falls_back_rather_than_reaching_the_mesh() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                (scalar(Some(value), DEFAULT_SIZE, 0.0..=MAX_SIZE) - DEFAULT_SIZE).abs() < 1.0e-9
            );
        }
    }

    #[test]
    fn an_absurd_length_is_held_to_the_range() {
        assert!((scalar(Some(1.0e30), DEFAULT_SIZE, 0.0..=MAX_SIZE) - MAX_SIZE).abs() < 1.0e-6);
        assert!(scalar(Some(-4.0), DEFAULT_SIZE, 0.0..=MAX_SIZE).abs() < 1.0e-9);
        assert!((scalar(None, DEFAULT_SIZE, 0.0..=MAX_SIZE) - DEFAULT_SIZE).abs() < 1.0e-9);
    }

    #[test]
    fn a_label_longer_than_the_cap_is_cut_on_a_character_boundary() {
        let value = "漢".repeat(MAX_CHARS + 32);
        let cut = truncated(&value);
        assert_eq!(cut.chars().count(), MAX_CHARS);
        assert_eq!(truncated("hello"), "hello");
    }
}
