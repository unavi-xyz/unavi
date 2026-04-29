use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use hsd::{Hsd, HsdNode};
use loro::{LoroMap, TreeID};
use loro_surgeon::{Hydrate, TreeNode};

use super::{diff::extract_changes_from_diff, events::RawChangeQueue};

use crate::{
    DocRegistryMap, HsdDoc, HsdEntityMaps, HsdRecordId, HsdSubscription,
    hydrate::events::RawHsdChange,
};

pub fn init_hsd_doc(
    mut commands: Commands,
    added: Query<(Entity, &HsdDoc, &HsdRecordId), (Added<HsdDoc>, Without<RawChangeQueue>)>,
    mut registry_map: ResMut<DocRegistryMap>,
) {
    for (doc_ent, hsd_doc, record_id) in &added {
        let doc = Arc::clone(&hsd_doc.0);

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
            HsdEntityMaps::default(),
            RawChangeQueue(raw_queue),
            HsdSubscription(sub),
        ));

        registry_map.0.insert(record_id.0, doc_ent);
    }
}

fn full_hydrate(hsd_map: &LoroMap, raw_queue: &Arc<Mutex<Vec<RawHsdChange>>>) {
    let hsd = Hsd::hydrate(&hsd_map.get_deep_value()).unwrap_or_default();
    let mut raw = raw_queue.lock().expect("raw queue lock");

    for id in hsd.images.keys() {
        raw.push(RawHsdChange::ImageAdded { id: id.clone() });
    }
    for id in hsd.materials.keys() {
        raw.push(RawHsdChange::MaterialAdded { id: id.clone() });
    }
    for id in hsd.meshes.keys() {
        raw.push(RawHsdChange::MeshAdded { id: id.clone() });
    }

    visit_nodes(&hsd.nodes, None, &mut raw);
}

fn visit_nodes(
    nodes: &[TreeNode<HsdNode>],
    parent_id: Option<TreeID>,
    raw: &mut Vec<RawHsdChange>,
) {
    for node in nodes {
        let Some(tree_id) = node.id else { continue };
        raw.push(RawHsdChange::NodeAdded { tree_id, parent_id });
        visit_nodes(&node.children, Some(tree_id), raw);
    }
}
