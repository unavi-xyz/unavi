use bevy::prelude::*;
use bevy_msdf::{
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

use crate::{
    attributes::values::{
        clamped,
        from_color_vec,
    },
    billboard::Billboard,
};

/// Em height in metres, body text at arm's length.
const DEFAULT_SIZE: f32 = 0.02;

/// Largest em height, in metres.
const MAX_SIZE: f32 = 100.0;

/// A cheap pre-trim on the raw character count, before a peer-controlled
/// string is even allocated into a [`SmolStr`]. `msdf::layout::layout` owns
/// the authoritative cap and reports `truncated` after tab expansion; this
/// only keeps an attacker's string from being held in full by the component.
const MAX_CHARS: usize = MAX_GLYPHS;

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
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
        width: clamped(attr.outline_width, 0.25, 0.0..=1.0),
    });

    commands.entity(entity).insert((
        MsdfText {
            value:       truncated(&attr.value),
            size:        clamped(attr.size, DEFAULT_SIZE, 0.0..=MAX_SIZE),
            align:       align(attr.align),
            anchor:      anchor(attr.anchor),
            wrap:        attr
                .wrap
                .map(|wrap| clamped(Some(wrap), 0.0, 0.0..=MAX_SIZE))
                .filter(|wrap| *wrap > 0.0),
            line_height: clamped(attr.line_height, 1.0, 0.0..=MAX_SIZE),
            font:        None,
        },
        MsdfStyle {
            color: from_color_vec(attr.color.as_ref(), Color::WHITE),
            outline,
            emissive: clamped(attr.emissive, 0.0, 0.0..=MAX_SIZE),
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
    fn a_label_longer_than_the_cap_is_cut_on_a_character_boundary() {
        let value = "漢".repeat(MAX_CHARS + 32);
        let cut = truncated(&value);
        assert_eq!(cut.chars().count(), MAX_CHARS);
        assert_eq!(truncated("hello"), "hello");
    }
}
