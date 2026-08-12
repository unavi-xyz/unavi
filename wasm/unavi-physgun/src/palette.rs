use wired_prelude::prelude::*;

pub const DEFAULT: Color = Color {
    r: 1.0,
    g: 0.62,
    b: 0.16,
    a: 1.0,
};

/// The tint the beam graph multiplies its output by; brightness lives in the
/// graph's own intensity input.
pub const fn beam_tint(color: Color) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: 1.0,
    }
}
