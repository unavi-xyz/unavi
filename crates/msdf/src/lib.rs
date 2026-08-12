//! Multi-channel signed distance field text: a field grown at runtime from a
//! parsed face, and laying a string out against one.
//!
//! Nothing here draws; a renderer builds whatever its pipeline wants from
//! [`layout::Laid`]'s quads, which keeps the layout testable without a GPU.

pub mod atlas;
pub mod font;
pub mod generate;
pub mod layout;
pub mod outline;
pub mod runtime;
