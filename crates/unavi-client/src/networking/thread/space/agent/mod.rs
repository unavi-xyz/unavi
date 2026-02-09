//! Agent networking - real-time pose streaming for avatars.
//!
//! Stream routing is handled by `SpaceProtocol` in the parent module.
//! This module provides individual stream handlers for agent data.

pub mod inbound;
pub mod outbound;
