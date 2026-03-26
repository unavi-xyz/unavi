use bevy::prelude::*;

/// Inbound event whitelist for a document. Absent = no restriction.
/// Only doc IDs in `allowed` may send events to this document.
#[derive(Component, Default)]
pub struct DocumentFirewall {
    pub allowed: Vec<blake3::Hash>,
}
