//! Spatial UI for UNAVI, exported as `unavi:vui` so any script can put up a
//! surface and drive it.
//!
//! Layout, targeting and interaction are host-testable modules of their own;
//! [`scene`] draws them into the calling script's document, and [`api`] is the
//! only thing a consumer sees.

pub mod api;
pub mod assist;
pub mod attention;
pub mod circle;
pub mod grasp;
pub mod layout;
pub mod mesh;
pub mod mote;
pub mod palette;
pub mod placard;
pub mod pointer;
pub mod scene;
pub mod surface;
pub mod tree;
pub mod tuning;
pub mod view;

wired_prelude::generate!();

struct World;

export!(World);
