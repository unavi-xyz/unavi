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
mod fit;
mod grasp;
mod layout;
mod mesh;
mod mote;
mod palette;
mod placard;
mod pointer;
mod render;
mod reveal;
mod surface;
mod tree;
mod tuning;
mod view;

wired_guest::generate!();

struct World;

export!(World);
