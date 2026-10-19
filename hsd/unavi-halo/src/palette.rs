//! Tints for halo's motes.
//!
//! VUI reserves colour for state and carries identity in the label, which is
//! right for the shell it draws. A tint is the one thing a consumer may add,
//! so this is where a slot gets to look like itself. Per-slot colour is most
//! of what makes a ring readable at a glance. The glyph inside takes its
//! colour from the tint, so nothing here colours the dots themselves.
//!
//! Each sits as bright as the colour a beacon is given (value 0.7 and up).
//! The glass lets only a fraction of its tint through, and a darker hue reads
//! as a dim, flat shell beside a lighter one.

use wired_guest::math::Color;

pub const HOME: Color = rgb(0.95, 0.11, 0.09);
pub const NAV: Color = rgb(0.10, 0.80, 0.35);
pub const TOOLS: Color = rgb(0.05, 0.38, 0.94);
pub const SELF: Color = rgb(0.85, 0.20, 0.90);

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
