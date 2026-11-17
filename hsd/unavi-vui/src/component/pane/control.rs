//! The controls a block holds, and what a press on each does.

use smol_str::SmolStr;

use crate::{
    component::pane::{
        COLUMN,
        Event,
    },
    face::measure::text,
};

/// Width of a toggle: two cells, the lit one showing the state.
pub const TOGGLE_EM: f32 = 2.4;
/// Space either side of an action's or a choice's label.
const PAD: f32 = COLUMN;

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    /// Text to read.
    Value(SmolStr),
    Action {
        label:   SmolStr,
        primary: bool,
    },
    Toggle(bool),
    /// A fraction from 0 to 1, dragged along its row.
    Slider(f32),
    /// A fraction from 0 to 1, read only.
    Meter(f32),
    /// Dragged sideways, `per_column` of value for each column of travel.
    Number {
        value:      f32,
        per_column: f32,
    },
    Choice {
        options:  Vec<SmolStr>,
        selected: usize,
    },
    Link(SmolStr),
}

/// What a press on something does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Press {
    Nothing,
    Act,
    /// Asks for this state.
    Toggle(bool),
    /// Asks for this fraction, and goes on asking while it is dragged.
    Slide(f32),
    /// Starts a drag on a number.
    Scrub {
        per_column: f32,
    },
    Choose(usize),
    Follow,
    Select,
    /// Asks for this state.
    Fold(bool),
    /// Starts a drag on a scroll rail, at this fraction of its length.
    Rail(f32),
}

impl Press {
    /// What a press reports at once. A drag reports as it moves.
    #[must_use]
    pub const fn event(self, id: u32) -> Option<Event> {
        match self {
            Self::Act => Some(Event::Pressed(id)),
            Self::Toggle(on) => Some(Event::Toggled(id, on)),
            Self::Slide(value) => Some(Event::Slid(id, value)),
            Self::Choose(index) => Some(Event::Chosen(id, index as u32)),
            Self::Follow => Some(Event::Followed(id)),
            Self::Select => Some(Event::Selected(id)),
            Self::Fold(open) => Some(Event::Folded(id, open)),
            Self::Nothing | Self::Scrub { .. } | Self::Rail(_) => None,
        }
    }
}

impl Control {
    /// Width in ems, within `room`.
    #[must_use]
    pub fn width(&self, room: f32) -> f32 {
        let natural = match self {
            Self::Value(text) | Self::Link(text) => text::width(text),
            Self::Action { label, .. } => text::width(label) + 2.0 * PAD,
            Self::Toggle(_) => TOGGLE_EM,
            Self::Slider(_) | Self::Meter(_) => room,
            Self::Number { value, .. } => text::width(&format_number(*value)),
            Self::Choice { options, .. } => segments(options).last().map_or(0.0, |s| s.0 + s.1),
        };
        natural.min(room)
    }

    /// What a press does `x` ems from the control's left edge.
    #[must_use]
    pub fn press(&self, x: f32, width: f32) -> Press {
        match self {
            Self::Value(_) | Self::Meter(_) => Press::Nothing,
            Self::Action { .. } => Press::Act,
            Self::Toggle(on) => Press::Toggle(!on),
            Self::Slider(_) => Press::Slide(if width > 0.0 {
                (x / width).clamp(0.0, 1.0)
            } else {
                0.0
            }),
            Self::Number { per_column, .. } => Press::Scrub {
                per_column: *per_column,
            },
            Self::Choice { options, .. } => segments(options)
                .iter()
                .position(|(start, width)| x >= *start && x < start + width)
                .map_or(Press::Nothing, Press::Choose),
            Self::Link(_) => Press::Follow,
        }
    }

    /// What the keyboard's Enter does on it.
    #[must_use]
    pub const fn activation(&self) -> Press {
        match self {
            Self::Action { .. } => Press::Act,
            Self::Toggle(on) => Press::Toggle(!*on),
            Self::Choice { options, selected } if !options.is_empty() => {
                Press::Choose((*selected + 1) % options.len())
            }
            Self::Link(_) => Press::Follow,
            _ => Press::Nothing,
        }
    }

    /// Whether a press on it does anything.
    #[must_use]
    pub const fn pressable(&self) -> bool {
        !matches!(self, Self::Value(_) | Self::Meter(_))
    }
}

/// The left edge and width of each option of a choice, in ems from its left.
#[must_use]
pub fn segments(options: &[SmolStr]) -> Vec<(f32, f32)> {
    let mut start = 0.0;
    options
        .iter()
        .map(|option| {
            let width = text::width(option) + 2.0 * PAD;
            let segment = (start, width);
            start += width;
            segment
        })
        .collect()
}

/// A number as it is drawn: up to three decimals, none of them trailing.
#[must_use]
pub fn format_number(value: f32) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" || text.is_empty() {
        return "0".to_owned();
    }
    text.to_owned()
}
