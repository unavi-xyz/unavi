//! Colours for the glyphs halo draws inside its motes.
//!
//! VUI reserves colour for state and carries identity in the label, which is
//! right for the shell it draws. An icon is the consumer's own prim, so this
//! is where a slot gets to look like itself. Per-slot colour is most of what
//! makes a ring readable at a glance.

use wired_prelude::prelude::*;

pub const HOME: Color = rgb(0.86, 0.10, 0.08);
pub const NAV: Color = rgb(0.06, 0.50, 0.22);
pub const TOOLS: Color = rgb(0.04, 0.30, 0.75);

/// What every glyph wears. A cool off-white reads as a glyph against the bold
/// shell and against the bright room alike, the way a light accent sits on a
/// coloured wall in Mirror's Edge — bold surface, light mark.
pub const GLYPH: Color = rgb(0.88, 0.90, 0.94);

/// A colour per tool, so two tools are told apart before either is read.
///
/// Indexed by the tool's place in a stably sorted list, so a tool keeps its
/// colour across sessions as long as the set does.
const TOOLS_WHEEL: [Color; 6] = [
    rgb(0.90, 0.55, 0.05),
    rgb(0.45, 0.30, 0.95),
    rgb(0.05, 0.65, 0.55),
    rgb(0.85, 0.25, 0.10),
    rgb(0.60, 0.65, 0.10),
    rgb(0.80, 0.15, 0.45),
];

#[must_use]
pub const fn tool(index: usize) -> Color {
    TOOLS_WHEEL[index % TOOLS_WHEEL.len()]
}

const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}
