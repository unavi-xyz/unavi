//! Render layer shared by every crate that draws or excludes a portal's
//! live-view plane (`unavi-agent`, `unavi-client`, this crate).
//!
//! Kept here, not duplicated, because both of those crates already depend
//! on `unavi-portal`.

/// Layer carrying a portal's rendered plane, kept off every first-person
/// camera layer set so a portal never shows in a mirror of itself.
pub const PORTAL_RENDER_LAYER: usize = 5;
