use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use loro::LoroMap;
use smol_str::ToSmolStr;

use super::{
    diff::extract_changes_from_diff,
    events::{DocEventQueue, HsdChangeQueue},
    node::{node_transform, spawn_node_entity},
};
use crate::{
    HsdChild, HsdDoc, HsdSubscription,
    cache::{
        MaterialInner, MaterialState, MeshInner, MeshState, NodeInner, NodeState, SceneRegistry,
        SceneRegistryInner,
    },
    data::{HsdMaterial, HsdMesh, hydrate_hsd},
};

pub fn init_hsd_doc(
    mut commands: Commands,
    added: Query<(Entity, &HsdDoc), (Added<HsdDoc>, Without<SceneRegistry>)>,
) {
    for (doc_ent, hsd_doc) in &added {
        let doc = Arc::clone(&hsd_doc.0);
        let registry = SceneRegistryInner::new();

        let hsd_map = doc.get_map("hsd");
        full_hydrate(&hsd_map, doc_ent, &mut commands, &registry);

        let change_queue: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(Vec::new()));
        let cq = Arc::clone(&change_queue);
        let sub = doc.subscribe_root(Arc::new(move |e| {
            if let Ok(mut locked) = cq.try_lock() {
                extract_changes_from_diff(&e, &mut locked);
            }
        }));

        commands.entity(doc_ent).insert((
            SceneRegistry(Arc::clone(&registry)),
            DocEventQueue(Arc::new(Mutex::new(Vec::new()))),
            HsdChangeQueue(change_queue),
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
            dirty: false.into(),
            id: id.clone(),
            state: Mutex::new(MaterialState::default()),
            entity: Mutex::new(Some(ent)),
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
            dirty: false.into(),
            id: id.clone(),
            state: Mutex::new(MeshState::default()),
            entity: Mutex::new(Some(ent)),
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
            ..Default::default()
        };
        let id = tree_id.to_smolstr();
        let inner = Arc::new(NodeInner {
            dirty: false.into(),
            entity: Mutex::new(None),
            id: id.clone(),
            state: Mutex::new(node_state),
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
