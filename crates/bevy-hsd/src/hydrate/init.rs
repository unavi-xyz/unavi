use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use loro::LoroMap;
use smol_str::ToSmolStr;

use super::{
    apply::raw_to_doc_change,
    diff::extract_changes_from_diff,
    events::{DocChange, DocChangeQueue},
    node::{node_transform, spawn_node_entity},
};
use crate::{
    HsdChild, HsdDoc, HsdSubscription,
    cache::{
        MaterialChanges, MaterialInner, MaterialState, MeshChanges, MeshInner, MeshState,
        NodeChanges, NodeInner, NodeState, SceneRegistry, SceneRegistryInner,
    },
    data::{HsdMaterial, HsdMesh, hydrate_hsd},
};

/// # Panics
///
/// Panics if the doc change queue poisons.
pub fn init_hsd_doc(
    mut commands: Commands,
    added: Query<(Entity, &HsdDoc), (Added<HsdDoc>, Without<SceneRegistry>)>,
) {
    for (doc_ent, hsd_doc) in &added {
        let doc = Arc::clone(&hsd_doc.0);
        let registry = SceneRegistryInner::new();

        let hsd_map = doc.get_map("hsd");
        full_hydrate(&hsd_map, doc_ent, &mut commands, &registry);

        let change_queue: Arc<Mutex<Vec<DocChange>>> = Arc::new(Mutex::new(Vec::new()));
        let cq = Arc::clone(&change_queue);
        let doc_arc = Arc::clone(&doc);
        let sub = doc.subscribe_root(Arc::new(move |e| {
            let mut raw = Vec::new();
            extract_changes_from_diff(&e, &mut raw);
            if raw.is_empty() {
                return;
            }
            let hsd_map = doc_arc.get_map("hsd");
            let changes: Vec<DocChange> = raw
                .into_iter()
                .filter_map(|r| raw_to_doc_change(doc_ent, r, &hsd_map))
                .collect();
            cq.lock().expect("change queue lock").extend(changes);
        }));

        commands.entity(doc_ent).insert((
            SceneRegistry(Arc::clone(&registry)),
            DocChangeQueue(change_queue),
            HsdSubscription(sub),
        ));
    }
}

pub(super) fn full_hydrate(
    hsd_map: &LoroMap,
    doc_ent: Entity,
    commands: &mut Commands,
    registry: &Arc<SceneRegistryInner>,
) {
    let hsd_data = match hydrate_hsd(hsd_map) {
        Ok(d) => d,
        Err(err) => {
            warn!(?err, "failed to hydrate hsd doc");
            return;
        }
    };

    for (id, mat) in &hsd_data.materials {
        let ent = commands
            .spawn((HsdChild { doc: doc_ent }, HsdMaterial::clone(mat)))
            .id();
        let inner = Arc::new(MaterialInner {
            changes: Mutex::new(MaterialChanges::default()),
            entity: Mutex::new(Some(ent)),
            id: id.clone(),
            state: Mutex::new(MaterialState::default()),
            sync: false.into(),
        });
        registry
            .materials
            .lock()
            .expect("materials lock")
            .insert(id.clone(), inner);
    }

    for (id, mesh) in &hsd_data.meshes {
        let ent = commands
            .spawn((HsdChild { doc: doc_ent }, HsdMesh::clone(mesh)))
            .id();
        let inner = Arc::new(MeshInner {
            changes: Mutex::new(MeshChanges::default()),
            entity: Mutex::new(Some(ent)),
            id: id.clone(),
            state: Mutex::new(MeshState::default()),
            sync: false.into(),
        });
        registry
            .meshes
            .lock()
            .expect("meshes lock")
            .insert(id.clone(), inner);
    }

    let mut id_to_inner = HashMap::new();

    for (tree_id, node) in &hsd_data.nodes {
        let node_state = NodeState {
            name: node
                .data
                .name
                .as_ref()
                .map(std::string::ToString::to_string),
            transform: node_transform(&node.data),
            mesh: node.data.mesh.clone(),
            material: node.data.material.clone(),
            collider: node.data.collider.clone(),
            rigid_body: node.data.rigid_body.clone(),
            ..Default::default()
        };
        let id = tree_id.to_smolstr();
        let inner = Arc::new(NodeInner {
            changes: Mutex::new(NodeChanges::default()),
            entity: Mutex::new(None),
            id: id.clone(),
            is_virtual: false,
            state: Mutex::new(node_state),
            sync: false.into(),
            tree_id: Mutex::new(Some(*tree_id)),
        });
        let ent = spawn_node_entity(doc_ent, node, commands);
        *inner.entity.lock().expect("entity lock") = Some(ent);
        registry
            .nodes
            .lock()
            .expect("nodes lock")
            .push(Arc::clone(&inner));
        registry
            .node_map
            .lock()
            .expect("node_map lock")
            .insert(id.clone(), Arc::clone(&inner));
        id_to_inner.insert(id, inner);
    }

    for node in hsd_data.nodes.values() {
        if let Some(parent_id) = &node.parent_id
            && let Some(child_inner) = id_to_inner.get(&node.id)
            && let Some(parent_inner) = id_to_inner.get(parent_id)
        {
            let child_ent = *child_inner.entity.lock().expect("entity lock");
            let parent_ent = *parent_inner.entity.lock().expect("entity lock");
            if let (Some(c), Some(p)) = (child_ent, parent_ent) {
                commands.entity(c).insert(ChildOf(p));
            }
        }
    }
}
