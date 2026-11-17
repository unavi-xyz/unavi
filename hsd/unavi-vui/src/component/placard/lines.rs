//! Fits a placard's text to a pane: wraps it into lines and places the pane
//! beside its body.

use arrayvec::ArrayVec;
use smol_str::SmolStr;
use wired_guest::math::{
    Vec2,
    Vec3,
};

use super::{
    Emphasis,
    MAX_LINES,
    Placard,
    PlacardLine,
    Tone,
    icon_advance,
};
use crate::tokens::tuning::Tuning;

/// Mounts a placard clear of the body at `mote`.
///
/// The panel hangs down from its own top edge, so the mount is a whole
/// `height` above the mote rather than resting on it — the two would otherwise
/// overlap for every placard that is more than a line long. Above rather than
/// beside, because the mote's own name is already below it and a placard that
/// picked a side would leave the surface at the edge slots.
pub(super) fn mount(mote: Vec3, radius: f32, height: f32, tuning: &Tuning) -> Vec3 {
    Vec3::new(
        mote.x,
        mote.y + radius + tuning.placard_gap + height,
        mote.z + tuning.placard_lift,
    )
}

/// Longest line, in characters, that fits `width` at `size`, estimated from
/// [`Tuning::advance_estimate`]. Only the renderer knows what a string
/// measures.
pub(super) fn budget(width: f32, size: f32, tuning: &Tuning) -> usize {
    let glyph = size * tuning.advance_estimate;
    if glyph <= f32::EPSILON {
        return usize::MAX;
    }
    ((width / glyph) as usize).max(1)
}

/// Greedy wrap at `budget` characters, breaking on spaces and newlines.
///
/// A word longer than the budget is split where it runs out of room.
pub(super) fn wrap(text: &str, budget: usize, out: &mut ArrayVec<SmolStr, MAX_LINES>) {
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let mut word = word;
            while word.chars().count() > budget {
                let (head, tail) = split_at_chars(word, budget);
                if !line.is_empty() {
                    push(out, &line);
                    line.clear();
                }
                push(out, head);
                word = tail;
            }
            let candidate =
                line.chars().count() + usize::from(!line.is_empty()) + word.chars().count();
            if !line.is_empty() && candidate > budget {
                push(out, &line);
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        push(out, &line);
    }
}

pub(super) fn push(out: &mut ArrayVec<SmolStr, MAX_LINES>, text: &str) {
    let _ = out.try_push(SmolStr::new(text));
}

pub(super) fn split_at_chars(text: &str, count: usize) -> (&str, &str) {
    let at = text
        .char_indices()
        .nth(count)
        .map_or(text.len(), |(index, _)| index);
    text.split_at(at)
}

pub(super) fn lay_out(
    placard: &Placard,
    tuning: &Tuning,
) -> (Vec2, ArrayVec<PlacardLine, MAX_LINES>) {
    let mut lines = ArrayVec::new();
    let inner = tuning.placard_pad.mul_add(-2.0, tuning.placard_width);
    let mut y = -tuning.placard_pad;

    if let Some(kicker) = placard
        .kicker
        .as_deref()
        .filter(|kicker| !kicker.is_empty())
    {
        let mut wrapped = ArrayVec::new();
        wrap(
            kicker,
            budget(inner, tuning.placard_kicker, tuning),
            &mut wrapped,
        );
        for text in wrapped {
            y -= tuning.placard_kicker;
            let _ = lines.try_push(PlacardLine {
                text,
                offset: Vec2::new(-inner / 2.0, y),
                size: tuning.placard_kicker,
                emphasis: Emphasis::Kicker,
                icon: None,
            });
        }
    }

    if !placard.title.is_empty() {
        let mut wrapped = ArrayVec::new();
        wrap(
            &placard.title,
            budget(inner, tuning.placard_title, tuning),
            &mut wrapped,
        );
        for text in wrapped {
            y -= tuning.placard_title;
            let _ = lines.try_push(PlacardLine {
                text,
                offset: Vec2::new(-inner / 2.0, y),
                size: tuning.placard_title,
                emphasis: if placard.warning {
                    Emphasis::Warning
                } else {
                    Emphasis::Title
                },
                icon: None,
            });
        }
    }

    let step = tuning.placard_row * tuning.placard_line;
    for paragraph in &placard.body {
        let indent = paragraph
            .icon
            .as_ref()
            .map_or(0.0, |_| icon_advance(tuning.placard_row));
        let mut wrapped = ArrayVec::new();
        wrap(
            &paragraph.text,
            budget(inner - indent, tuning.placard_row, tuning),
            &mut wrapped,
        );
        for (index, text) in wrapped.into_iter().enumerate() {
            y -= step;
            let _ = lines.try_push(PlacardLine {
                text,
                offset: Vec2::new(-inner / 2.0 + indent, y),
                size: tuning.placard_row,
                emphasis: match paragraph.tone {
                    Tone::Body => Emphasis::Body,
                    Tone::Muted => Emphasis::Muted,
                    Tone::Warning => Emphasis::Warning,
                },
                icon: paragraph.icon.clone().filter(|_| index == 0),
            });
        }
    }

    (
        Vec2::new(tuning.placard_width, -y + tuning.placard_pad),
        lines,
    )
}
