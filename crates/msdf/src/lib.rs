//! Multi-channel signed distance field text: the baked atlas format, and
//! laying a string out against one.
//!
//! Nothing here draws. A renderer takes [`layout::Laid`] and builds whatever
//! its pipeline wants from the quads, which is what keeps the interesting part
//! — metrics, wrapping, alignment — testable without a GPU.

pub mod atlas;
#[cfg(feature = "bake")] pub mod bake;
pub mod layout;
