use std::sync::{Arc, Mutex};

use bevy::prelude::*;

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
    hsd_map: &loro::LoroMap,
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

    let mut tree_id_to_inner: std::collections::HashMap<String, Arc<NodeInner>> =
        std::collections::HashMap::new();

    for node in &hsd_data.nodes {
        let Ok(tid) = loro::TreeID::try_from(node.tree_id.as_str()) else {
            continue;
        };
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
        let inner = Arc::new(NodeInner {
            dirty: false.into(),
            tree_id: tid,
            state: Mutex::new(node_state),
            entity: Mutex::new(None),
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
            .insert(tid, Arc::clone(&inner));
        tree_id_to_inner.insert(node.tree_id.clone(), inner);
    }

    for node in &hsd_data.nodes {
        if let Some(parent_id) = &node.parent_tree_id
            && let Some(child_inner) = tree_id_to_inner.get(&node.tree_id)
            && let Some(parent_inner) = tree_id_to_inner.get(parent_id)
        {
            let child_ent = *child_inner.entity.lock().expect("entity lock");
            let parent_ent = *parent_inner.entity.lock().expect("entity lock");
            if let (Some(c), Some(p)) = (child_ent, parent_ent) {
                commands.entity(c).insert(ChildOf(p));
            }
        }
    }
}
