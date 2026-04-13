//! One-shot setup for new HSD documents: initial full hydration + subscription.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use loro::{LoroMap, LoroTree, LoroValue, TreeParentId};

use super::{diff::extract_changes_from_diff, events::RawChangeQueue};

use crate::{
    DocRegistryMap, HsdDoc, HsdRecordId, HsdSubscription,
    cache::{SceneRegistry, SceneRegistryInner},
    hydrate::events::RawHsdChange,
};

pub fn init_hsd_doc(
    mut commands: Commands,
    added: Query<(Entity, &HsdDoc), (Added<HsdDoc>, Without<SceneRegistry>)>,
) {
    for (doc_ent, hsd_doc) in &added {
        let doc = Arc::clone(&hsd_doc.0);
        let registry = SceneRegistryInner::new();

        let raw_queue: Arc<Mutex<Vec<RawHsdChange>>> = Arc::new(Mutex::new(Vec::new()));

        let hsd_map = doc.get_map("hsd");
        full_hydrate(&hsd_map, &raw_queue);

        let rq = Arc::clone(&raw_queue);
        let sub = doc.subscribe_root(Arc::new(move |e| {
            let mut raw = Vec::new();
            extract_changes_from_diff(&e, &mut raw);
            if !raw.is_empty() {
                rq.lock().expect("raw queue lock").extend(raw);
            }
        }));

        commands.entity(doc_ent).insert((
            SceneRegistry(Arc::clone(&registry)),
            RawChangeQueue(raw_queue),
            HsdSubscription(sub),
        ));
    }
}

/// Registers newly initialized doc entities in the global [`DocRegistryMap`].
///
/// Runs after `init_hsd_doc` so `SceneRegistry` is always present when this fires.
/// Covers both WDS docs (registry added by `init_hsd_doc`) and script-created
/// docs (registry present from spawn time).
pub fn register_doc_registries(
    added: Query<(Entity, &HsdRecordId, &SceneRegistry), Added<SceneRegistry>>,
    mut map: ResMut<DocRegistryMap>,
) {
    for (entity, record_id, registry) in &added {
        map.0.insert(record_id.0, (entity, Arc::clone(&registry.0)));
    }
}

/// Emits synthetic `*Added` events for all objects already in the document so
/// the queue processor handles initial load identically to incremental diffs.
/// Images are emitted before materials because materials reference image IDs.
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
