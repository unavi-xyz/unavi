use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use loro::{ExportMode, LoroDoc, LoroMap, LoroValue};
use loro_surgeon::Hydrate;

use super::diff::extract_changes_from_diff;
use super::node::{spawn_node_entity, spawn_node_from_overlay, update_node_components};
use crate::{
    HsdBaseSubscription, HsdChange, HsdChild, HsdDoc, HsdEntityMap, HsdEventQueue,
    HsdScriptOverlay, HsdSubscription,
    data::{HsdMaterial, HsdMesh, hydrate_hsd},
};

pub fn init_script_overlay(mut commands: Commands, added: Query<(Entity, &HsdDoc), Added<HsdDoc>>) {
    for (ent, base) in &added {
        let overlay = Arc::new(LoroDoc::new());

        match base.0.export(ExportMode::all_updates()) {
            Ok(bytes) if !bytes.is_empty() => {
                if let Err(e) = overlay.import(&bytes) {
                    warn!(?e, "failed to import base into overlay");
                }
            }
            Err(e) => warn!(?e, "failed to export base for overlay"),
            _ => {}
        }

        let overlay_clone = Arc::clone(&overlay);
        let base_clone = Arc::clone(&base.0);
        let base_sub = base.0.subscribe_root(Arc::new(move |_e| {
            if let Ok(bytes) = base_clone.export(ExportMode::all_updates()) {
                overlay_clone.import(&bytes).ok();
            }
        }));

        commands
            .entity(ent)
            .insert((HsdScriptOverlay(overlay), HsdBaseSubscription(base_sub)));
    }
}

pub fn init_hsd_doc(
    mut commands: Commands,
    added: Query<(Entity, &HsdScriptOverlay), Added<HsdScriptOverlay>>,
) {
    for (doc_ent, overlay) in &added {
        let entity_map = full_hydrate(&overlay.0, doc_ent, &mut commands);

        let queue: Arc<Mutex<Vec<HsdChange>>> = Arc::new(Mutex::new(Vec::new()));
        let q = Arc::clone(&queue);
        let sub = overlay.0.subscribe_root(Arc::new(move |e| {
            if let Ok(mut locked) = q.try_lock() {
                extract_changes_from_diff(&e, &mut locked);
            }
        }));

        commands
            .entity(doc_ent)
            .insert((entity_map, HsdEventQueue(queue), HsdSubscription(sub)));
    }
}

/// # Panics
///
/// Panics if the event queue mutex is poisoned.
pub fn apply_hsd_events(
    mut commands: Commands,
    mut docs: Query<(Entity, &HsdScriptOverlay, &mut HsdEntityMap, &HsdEventQueue)>,
) {
    for (doc_ent, overlay, mut map, queue) in &mut docs {
        let changes: Vec<HsdChange> = {
            let mut locked = queue.0.lock().expect("queue lock");
            locked.drain(..).collect()
        };

        if changes.is_empty() {
            continue;
        }

        let hsd_map = overlay.0.get_map("hsd");

        for change in changes {
            match change {
                HsdChange::NodeAdded { tree_id, parent_id } => {
                    if let Some(ent) =
                        spawn_node_from_overlay(doc_ent, &tree_id, &hsd_map, &mut commands)
                    {
                        if let Some(pid) = &parent_id
                            && let Some(&parent_ent) = map.nodes.get(pid.as_str())
                        {
                            commands.entity(ent).insert(ChildOf(parent_ent));
                        }
                        map.nodes.insert(tree_id, ent);
                    }
                }

                HsdChange::NodeRemoved { tree_id } => {
                    if let Some(ent) = map.nodes.remove(&tree_id) {
                        commands.entity(ent).despawn();
                    }
                }

                HsdChange::NodeMetaChanged { tree_id } => {
                    if let Some(&ent) = map.nodes.get(tree_id.as_str()) {
                        update_node_components(ent, &tree_id, &hsd_map, &mut commands);
                    }
                }

                HsdChange::MeshAdded => {
                    let index = map.meshes.len();
                    match get_mesh_at(&hsd_map, index) {
                        Some(mesh) => {
                            let ent = commands.spawn((HsdChild { doc: doc_ent }, mesh)).id();
                            map.meshes.push(Some(ent));
                        }
                        None => map.meshes.push(None),
                    }
                }

                HsdChange::MeshRemoved { index } => {
                    if let Some(slot) = map.meshes.get_mut(index)
                        && let Some(ent) = slot.take()
                    {
                        commands.entity(ent).despawn();
                    }
                }

                HsdChange::MeshChanged { index } => {
                    let old = map
                        .meshes
                        .get_mut(index)
                        .and_then(std::option::Option::take);
                    if let Some(ent) = old {
                        commands.entity(ent).despawn();
                    }
                    if let Some(mesh) = get_mesh_at(&hsd_map, index) {
                        let ent = commands.spawn((HsdChild { doc: doc_ent }, mesh)).id();
                        if let Some(slot) = map.meshes.get_mut(index) {
                            *slot = Some(ent);
                        }
                    }
                }

                HsdChange::MaterialAdded => {
                    let index = map.materials.len();
                    match get_material_at(&hsd_map, index) {
                        Some(mat) => {
                            let ent = commands.spawn((HsdChild { doc: doc_ent }, mat)).id();
                            map.materials.push(Some(ent));
                        }
                        None => map.materials.push(None),
                    }
                }

                HsdChange::MaterialRemoved { index } => {
                    if let Some(slot) = map.materials.get_mut(index)
                        && let Some(ent) = slot.take()
                    {
                        commands.entity(ent).despawn();
                    }
                }

                HsdChange::MaterialChanged { index } => {
                    let old = map
                        .materials
                        .get_mut(index)
                        .and_then(std::option::Option::take);
                    if let Some(ent) = old {
                        commands.entity(ent).despawn();
                    }
                    if let Some(mat) = get_material_at(&hsd_map, index) {
                        let ent = commands.spawn((HsdChild { doc: doc_ent }, mat)).id();
                        if let Some(slot) = map.materials.get_mut(index) {
                            *slot = Some(ent);
                        }
                    }
                }
            }
        }
    }
}

fn full_hydrate(overlay: &LoroDoc, doc_ent: Entity, commands: &mut Commands) -> HsdEntityMap {
    let mut entity_map = HsdEntityMap::default();

    let hsd_map = overlay.get_map("hsd");
    let hsd_data = match hydrate_hsd(&hsd_map) {
        Ok(d) => d,
        Err(err) => {
            warn!(?err, "failed to hydrate hsd doc");
            return entity_map;
        }
    };

    for mat in &hsd_data.materials {
        let ent = commands
            .spawn((HsdChild { doc: doc_ent }, HsdMaterial::clone(mat)))
            .id();
        entity_map.materials.push(Some(ent));
    }

    for mesh in &hsd_data.meshes {
        let ent = commands
            .spawn((HsdChild { doc: doc_ent }, HsdMesh::clone(mesh)))
            .id();
        entity_map.meshes.push(Some(ent));
    }

    let mut tree_to_ent: HashMap<String, Entity> = HashMap::new();

    for node in &hsd_data.nodes {
        let ent = spawn_node_entity(doc_ent, node, commands);
        tree_to_ent.insert(node.tree_id.clone(), ent);
        entity_map.nodes.insert(node.tree_id.as_str().into(), ent);
    }

    for node in &hsd_data.nodes {
        if let Some(parent_id) = &node.parent_tree_id
            && let Some(&parent_ent) = tree_to_ent.get(parent_id)
            && let Some(&child_ent) = tree_to_ent.get(&node.tree_id)
        {
            commands.entity(child_ent).insert(ChildOf(parent_ent));
        }
    }

    entity_map
}

fn get_mesh_at(hsd_map: &LoroMap, index: usize) -> Option<HsdMesh> {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return None;
    };
    let LoroValue::List(list) = root.get("meshes")? else {
        return None;
    };
    HsdMesh::hydrate(list.get(index)?).ok()
}

fn get_material_at(hsd_map: &LoroMap, index: usize) -> Option<HsdMaterial> {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return None;
    };
    let LoroValue::List(list) = root.get("materials")? else {
        return None;
    };
    HsdMaterial::hydrate(list.get(index)?).ok()
}
