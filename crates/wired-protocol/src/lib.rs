//! Wired protocol types and constants.
//!
//! This crate provides auto-generated Rust types from JSON schemas in the
//! Wired protocol, along with protocol and schema URL constants.

// Generated types from JSON schemas.
#[allow(clippy::all)]
mod types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

// Generated schema URL constants.
#[allow(clippy::all)]
mod schemas {
    include!(concat!(env!("OUT_DIR"), "/schemas.rs"));
}

// Generated protocol URL constants and embedded definitions.
#[allow(clippy::all)]
mod protocols {
    include!(concat!(env!("OUT_DIR"), "/protocols.rs"));
}

pub use protocols::*;
pub use schemas::*;
pub use types::*;
