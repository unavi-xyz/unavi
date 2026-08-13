//! Colours for the glyphs halo draws inside its motes.
//!
//! VUI reserves colour for state and carries identity in the label, which is
//! right for the shell it draws. An icon is the consumer's own prim, so this
//! is where a slot gets to look like itself. Per-slot colour is most of what
//! makes a ring readable at a glance.

use wired_prelude::prelude::*;

pub const HOME: Color = rgb(0.96, 0.20, 0.16);
pub const NAV: Color = rgb(0.42, 0.90, 0.44);
pub const TOOLS: Color = rgb(0.24, 0.72, 1.0);

/// A colour per tool, so two tools are told apart before either is read.
///
/// Indexed by the tool's place in a stably sorted list, so a tool keeps its
/// colour across sessions as long as the set does.
const TOOLS_WHEEL: [Color; 6] = [
    rgb(0.98, 0.76, 0.24),
    rgb(0.62, 0.55, 0.98),
    rgb(0.34, 0.86, 0.78),
    rgb(0.96, 0.52, 0.36),
    rgb(0.80, 0.86, 0.40),
    rgb(0.94, 0.46, 0.72),
];

#[must_use]
pub const fn tool(index: usize) -> Color {
    TOOLS_WHEEL[index % TOOLS_WHEEL.len()]
}

const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}
