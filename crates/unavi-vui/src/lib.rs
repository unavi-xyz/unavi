//! Spatial UI primitives for UNAVI, free of host bindings so the layout,
//! targeting and interaction logic is unit-testable on the host target. Guest
//! scripts drive prims from the values these modules compute.

pub mod assist;
pub mod attention;
pub mod grasp;
pub mod layout;
pub mod mesh;
pub mod mote;
pub mod palette;
pub mod tree;
pub mod tuning;
pub mod view;
