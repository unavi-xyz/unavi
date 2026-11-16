//! Spatial UI for UNAVI, exported as `unavi:vui` so any script can put up a
//! surface and drive it.
//!
//! Layout, targeting and interaction are host-testable modules of their own;
//! [`render`] draws them into the calling script's document, and [`api`] is
//! the only thing a consumer sees.

mod api;
mod assist;
mod attention;
mod cast;
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "no surface draws cell art yet")
)]
mod cells;
mod fit;
mod flip;
mod font;
mod glyph;
mod grasp;
mod layout;
mod marks;
mod measure;
mod mesh;
mod mote;
mod palette;
mod placard;
mod pointer;
mod render;
mod reveal;
mod slate;
mod surface;
mod tether;
mod tree;
mod tuning;
mod view;

wired_guest::generate!();

struct World;

export!(World);
