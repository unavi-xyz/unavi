use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use loro::{LoroTree, LoroValue, TreeParentId};

use super::{
    diff::extract_changes_from_diff,
    events::{RawChangeQueue, ScriptEventQueue},
};
use crate::{HsdDoc, HsdSubscription, cache::SceneRegistry};

#[expect(clippy::missing_panics_doc)]
pub fn init_hsd_doc(
    mut commands: Commands,
    added: Query<(Entity, &HsdDoc), (Added<HsdDoc>, Without<SceneRegistry>)>,
) {
    for (doc_ent, hsd_doc) in &added {
        let doc = Arc::clone(&hsd_doc.0);
        let registry = crate::cache::SceneRegistryInner::new();

        let raw_queue: Arc<Mutex<Vec<super::events::RawHsdChange>>> =
            Arc::new(Mutex::new(Vec::new()));
        let script_queue: Arc<Mutex<Vec<super::events::ScriptQueuedEvent>>> =
            Arc::new(Mutex::new(Vec::new()));

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
            ScriptEventQueue(script_queue),
            HsdSubscription(sub),
        ));
    }
}

fn full_hydrate(hsd_map: &loro::LoroMap, raw_queue: &Arc<Mutex<Vec<super::events::RawHsdChange>>>) {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else { return };

    let mut raw = raw_queue.lock().expect("raw queue lock");

    if let Some(LoroValue::Map(mats)) = root.get("materials") {
        for id in mats.keys() {
            raw.push(super::events::RawHsdChange::MaterialAdded {
                id: id.as_str().into(),
            });
        }
    }

    if let Some(LoroValue::Map(meshes)) = root.get("meshes") {
        for id in meshes.keys() {
            raw.push(super::events::RawHsdChange::MeshAdded {
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
            raw.push(super::events::RawHsdChange::NodeAdded {
                tree_id: node.id,
                parent_id,
            });
        }
    }
}
