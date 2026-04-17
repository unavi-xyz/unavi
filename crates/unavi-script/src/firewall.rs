use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use blake3::Hash;

/// Whether a capability is open to all, or restricted to a specific set of document IDs.
#[derive(Clone, Debug, Default)]
pub enum Access {
    #[default]
    Open,
    /// Only the listed document IDs are permitted. An empty set permits nobody.
    Restricted(HashSet<Hash>),
}

impl Access {
    pub fn permits(&self, id: &Hash) -> bool {
        match self {
            Access::Open => true,
            Access::Restricted(set) => set.contains(id),
        }
    }
}

/// Per-capability inbound firewall for a document.
///
/// Shared between the ECS world and WASM runtime threads via `Arc<RwLock<_>>`.
#[derive(Debug)]
pub struct HsdFirewallInner {
    /// Which document IDs may emit events into this document.
    pub event_receive: Access,
    /// Which document IDs may read this document's scene data.
    pub scene_read: Access,
    /// Which document IDs may write to this document's scene data.
    pub scene_write: Access,
}

impl HsdFirewallInner {
    /// Default for a space or unregistered doc: open events and reads, blocked writes.
    pub fn default_space() -> Self {
        Self {
            event_receive: Access::Open,
            scene_read: Access::Open,
            scene_write: Access::Restricted(HashSet::new()),
        }
    }

    /// For a script-created child doc: open events and reads, creator-only writes.
    pub fn for_child_doc(creator_id: Hash) -> Self {
        Self {
            event_receive: Access::Open,
            scene_read: Access::Open,
            scene_write: Access::Restricted(HashSet::from([creator_id])),
        }
    }
}

/// ECS component wrapping the shared firewall inner.
#[derive(Component, Clone)]
pub struct HsdFirewall(pub Arc<RwLock<HsdFirewallInner>>);

impl Default for HsdFirewall {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(HsdFirewallInner::default_space())))
    }
}

/// Entity-reference variant of [`Access`], used in [`HsdFirewallEntities`].
#[derive(Clone, Debug, Default)]
pub enum AccessEntities {
    /// No entity-driven override — leaves the inner capability unchanged.
    #[default]
    Open,
    /// Resolve these entities to doc IDs and replace the capability's set each tick.
    Restricted(Vec<Entity>),
}

/// Automatically populate a firewall's `Restricted` sets from entity references.
///
/// `Open` variants are no-ops. `Restricted` variants replace (not accumulate)
/// the corresponding inner capability each tick.
#[derive(Component, Default)]
#[require(HsdFirewall)]
pub struct HsdFirewallEntities {
    pub event_receive: AccessEntities,
    pub scene_read: AccessEntities,
    pub scene_write: AccessEntities,
}

pub fn sync_hsd_firewall_entities(
    documents: Query<&HsdRecordId>,
    firewalls: Query<(&HsdFirewallEntities, &HsdFirewall)>,
) {
    for (ents, fw) in &firewalls {
        let Ok(mut fw) = fw.0.try_write() else {
            continue;
        };
        sync_capability(&documents, &ents.event_receive, &mut fw.event_receive);
        sync_capability(&documents, &ents.scene_read, &mut fw.scene_read);
        sync_capability(&documents, &ents.scene_write, &mut fw.scene_write);
    }
}

fn sync_capability(
    documents: &Query<&HsdRecordId>,
    src: &AccessEntities,
    dst: &mut Access,
) {
    let AccessEntities::Restricted(entities) = src else {
        return;
    };
    let set = entities
        .iter()
        .filter_map(|&ent| documents.get(ent).ok().map(|id| id.0))
        .collect();
    *dst = Access::Restricted(set);
}
