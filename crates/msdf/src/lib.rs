//! Multi-channel signed distance field text: the baked atlas format, and
//! laying a string out against one.
//!
//! Nothing here draws; a renderer builds whatever its pipeline wants from
//! [`layout::Laid`]'s quads, which keeps the layout testable without a GPU.

pub mod atlas;
#[cfg(feature = "bake")] pub mod bake;
pub mod layout;
