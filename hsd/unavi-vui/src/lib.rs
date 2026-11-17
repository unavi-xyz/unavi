//! Spatial UI for UNAVI, exported as `unavi:vui` so any script can put up a
//! surface and drive it.
//!
//! # Layers
//!
//! Each layer uses only the ones before it: [`tokens`], [`material`],
//! [`part`], [`face`], [`machinery`], [`component`], then [`api`], the only
//! thing a consumer sees.

mod api;
mod component;
mod face;
mod machinery;
mod material;
mod part;
mod tokens;

wired_guest::generate!();

struct World;

export!(World);
