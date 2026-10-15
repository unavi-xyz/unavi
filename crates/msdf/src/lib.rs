//! Multi-channel signed distance field text: a field grown at runtime from a
//! parsed face, and laying a string out against one.
//!
//! Nothing here draws; a renderer builds whatever its pipeline wants from
//! [`layout::Layout`]'s quads.
//!
//! Layout is per-character: a line breaks left-to-right, wrapping between
//! words or, for CJK, between any two characters. There is no complex-script
//! shaping, joining, ligatures or bidi — Arabic, Indic and other
//! context-dependent scripts set each character in isolation and right-to-left
//! text draws in logical rather than visual order. `harfrust` is used only to
//! read pair-kerning out of a face's GPOS table, not to shape runs.

pub mod atlas;
pub mod field;
pub mod font;
pub mod glyph;
pub mod layout;
pub mod outline;
