use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use blake3::Hash;

/// Inbound whitelist for a document.
/// Limits all cross-HSD access — such as events or scene resources.
#[derive(Component, Clone, Default)]
pub struct HsdFirewall(pub Arc<RwLock<HsdFirewallInner>>);

#[derive(Default, Debug)]
pub struct HsdFirewallInner {
    pub read: HashSet<Hash>,
    pub write: HashSet<Hash>,
}

/// Automatically populate a firewall from documents attached to select entities.
#[derive(Component, Default)]
#[require(HsdFirewall)]
pub struct HsdFirewallEntities {
    pub read: Vec<Entity>,
    pub write: Vec<Entity>,
}

pub fn sync_hsd_firewall_entities(
    documents: Query<&HsdRecordId>,
    firewalls: Query<(&HsdFirewallEntities, &HsdFirewall)>,
) {
    for (ents, fw) in firewalls {
        let Ok(mut fw) = fw.0.try_write() else {
            continue;
        };

        for ent in &ents.read {
            let Ok(id) = documents.get(*ent) else {
                continue;
            };
            fw.read.insert(id.0);
        }

        for ent in &ents.write {
            let Ok(id) = documents.get(*ent) else {
                continue;
            };
            fw.write.insert(id.0);
        }
    }
}
