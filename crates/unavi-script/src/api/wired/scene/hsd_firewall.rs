use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use blake3::Hash;

#[derive(Default)]
pub struct HsdFirewallInner {
    pub read: Vec<Hash>,
    pub write: Vec<Hash>,
}

/// Per-document HSD access firewall. Declares which external doc IDs may read
/// or write this document's scene (nodes, meshes, materials).
///
/// Inner `Arc` is shared into `DocHandle.hsd_fw` so Bevy systems can update
/// permissions in-place without replacing the component.
#[derive(Component, Clone, Default)]
pub struct HsdFirewall(pub Arc<RwLock<HsdFirewallInner>>);
