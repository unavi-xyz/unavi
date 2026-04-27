use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use loro::{LoroMap, LoroTree, LoroValue, TreeParentId};

use super::{diff::extract_changes_from_diff, events::RawChangeQueue};

use crate::{
    DocRegistryMap, HsdAssets, HsdDoc, HsdEntityMaps, HsdRecordId, HsdSubscription,
    hydrate::events::RawHsdChange,
    load_hsd::read_hsd_assets,
};

pub fn init_hsd_doc(
    mut commands: Commands,
    added: Query<
        (Entity, &HsdDoc, &HsdRecordId),
        (Added<HsdDoc>, Without<RawChangeQueue>),
    >,
    mut registry_map: ResMut<DocRegistryMap>,
) {
    for (doc_ent, hsd_doc, record_id) in &added {
        let doc = Arc::clone(&hsd_doc.0);

        let raw_queue: Arc<Mutex<Vec<RawHsdChange>>> = Arc::new(Mutex::new(Vec::new()));

        let hsd_map = doc.get_map("hsd");
        full_hydrate(&hsd_map, &raw_queue);
        let assets = HsdAssets(read_hsd_assets(&hsd_map));

        let rq = Arc::clone(&raw_queue);
        let sub = doc.subscribe_root(Arc::new(move |e| {
            let mut raw = Vec::new();
            extract_changes_from_diff(&e, &mut raw);
            if !raw.is_empty() {
                rq.lock().expect("raw queue lock").extend(raw);
            }
        }));

        commands
            .entity(doc_ent)
            .insert((assets, HsdEntityMaps::default(), RawChangeQueue(raw_queue), HsdSubscription(sub)));

        registry_map.0.insert(record_id.0, doc_ent);
    }
}

fn full_hydrate(hsd_map: &LoroMap, raw_queue: &Arc<Mutex<Vec<RawHsdChange>>>) {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else { return };

    let mut raw = raw_queue.lock().expect("raw queue lock");

    if let Some(LoroValue::Map(images)) = root.get("images") {
        for id in images.keys() {
            raw.push(RawHsdChange::ImageAdded {
                id: id.as_str().into(),
            });
        }
    }

    if let Some(LoroValue::Map(mats)) = root.get("materials") {
        for id in mats.keys() {
            raw.push(RawHsdChange::MaterialAdded {
                id: id.as_str().into(),
            });
        }
    }

    if let Some(LoroValue::Map(meshes)) = root.get("meshes") {
        for id in meshes.keys() {
            raw.push(RawHsdChange::MeshAdded {
                id: id.as_str().into(),
            });
        }
    }

    if let Ok(tree) = hsd_map.get_or_create_container("nodes", LoroTree::new()) {
        for node in &tree.get_nodes(false) {
            let parent_id = match node.parent {
                TreeParentId::Node(pid) => Some(pid),
                _ => None,
            };
            raw.push(RawHsdChange::NodeAdded {
                tree_id: node.id,
                parent_id,
            });
        }
    }
}
