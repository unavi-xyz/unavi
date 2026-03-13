use std::collections::hash_map::Entry;
use std::ops::Deref;
use std::sync::{Arc, Mutex, atomic::Ordering};

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobResponse};
use bytemuck::cast_slice;
use bytes::Bytes;
use loro::{LoroMap, LoroTree, LoroValue, TreeID};
use loro_surgeon::Hydrate;
use smol_str::{SmolStr, ToSmolStr};
use wired_schemas::HydratedHash;

use super::{
    events::{
        DocChange, DocChangeKind, DocEventQueue, HsdChangeQueue, MaterialData, MeshData,
        RawHsdChange,
    },
    node::{node_transform, spawn_node_entity, update_node_components},
};
use crate::{
    HsdChild, HsdDoc,
    cache::{
        MaterialInner, MaterialState, MeshInner, MeshState, NodeInner, NodeState, SceneRegistry,
        SceneRegistryInner,
    },
    compile::{
        material::MaterialParams,
        mesh::{MeshAttrName, MeshParams},
    },
    data::{HsdMaterial, HsdMesh, HsdNode, HsdNodeData},
};

pub(crate) fn flush_scene_dirty(
    docs: Query<(Entity, &SceneRegistry)>,
    mut writer: MessageWriter<DocChange>,
) {
    for (doc_ent, registry) in &docs {
        {
            let nodes = registry.0.nodes.lock().expect("nodes lock");
            for inner in nodes.iter() {
                if inner.dirty.swap(false, Ordering::Relaxed) {
                    let (tree_id, data) = node_data_from_inner(inner);
                    writer.write(DocChange {
                        doc: doc_ent,
                        kind: DocChangeKind::NodeChanged { id: tree_id, data },
                    });
                }
            }
        }
        {
            let meshes = registry.0.meshes.lock().expect("meshes lock");
            for (id, inner) in meshes.iter() {
                debug_assert_eq!(*id, inner.id);
                if inner.dirty.swap(false, Ordering::Relaxed) {
                    let state = inner.state.lock().expect("mesh state lock").clone();
                    writer.write(DocChange {
                        doc: doc_ent,
                        kind: DocChangeKind::MeshChanged {
                            id: id.clone(),
                            data: MeshData::Inline(state),
                        },
                    });
                }
            }
        }
        {
            let materials = registry.0.materials.lock().expect("materials lock");
            for (id, inner) in materials.iter() {
                debug_assert_eq!(*id, inner.id);
                if inner.dirty.swap(false, Ordering::Relaxed) {
                    let state = inner.state.lock().expect("material state lock").clone();
                    writer.write(DocChange {
                        doc: doc_ent,
                        kind: DocChangeKind::MaterialChanged {
                            id: id.clone(),
                            data: MaterialData::Inline(state),
                        },
                    });
                }
            }
        }
    }
}

pub(crate) fn drain_all_changes(
    docs: Query<(
        Entity,
        &HsdDoc,
        Option<&HsdChangeQueue>,
        Option<&DocEventQueue>,
    )>,
    mut writer: MessageWriter<DocChange>,
) {
    for (doc_ent, hsd_doc, hsd_queue, event_queue) in &docs {
        // Drain script-event queue first so gen_id entities exist before Loro events.
        if let Some(eq) = event_queue {
            let events: Vec<DocChange> = {
                let mut locked = eq.0.lock().expect("doc event queue lock");
                locked.drain(..).collect()
            };
            for event in events {
                writer.write(event);
            }
        }

        // Drain Loro-subscription queue.
        if let Some(cq) = hsd_queue {
            let raw: Vec<RawHsdChange> = {
                let mut locked = cq.0.lock().expect("hsd change queue lock");
                locked.drain(..).collect()
            };
            let hsd_map = hsd_doc.0.get_map("hsd");
            for change in raw {
                if let Some(doc_change) = raw_to_doc_change(doc_ent, change, &hsd_map) {
                    writer.write(doc_change);
                }
            }
        }
    }
}

fn raw_to_doc_change(
    doc_ent: Entity,
    change: RawHsdChange,
    hsd_map: &LoroMap,
) -> Option<DocChange> {
    let kind = match change {
        RawHsdChange::NodeAdded { tree_id, parent_id } => {
            let data = node_data_from_hsd(hsd_map, tree_id);
            DocChangeKind::NodeAdded {
                id: tree_id.to_smolstr(),
                parent_id: parent_id.map(|p| p.to_smolstr()),
                data,
            }
        }
        RawHsdChange::NodeChanged { tree_id } => {
            let data = node_data_from_hsd(hsd_map, tree_id);
            DocChangeKind::NodeChanged {
                id: tree_id.to_smolstr(),
                data,
            }
        }
        RawHsdChange::NodeRemoved { tree_id } => DocChangeKind::NodeRemoved {
            id: tree_id.to_smolstr(),
        },
        RawHsdChange::MeshAdded { id } => {
            let data = get_mesh_at(hsd_map, &id).map(MeshData::Hsd)?;
            DocChangeKind::MeshAdded { id, data }
        }
        RawHsdChange::MeshChanged { id } => {
            let data = get_mesh_at(hsd_map, &id).map(MeshData::Hsd)?;
            DocChangeKind::MeshChanged { id, data }
        }
        RawHsdChange::MeshRemoved { id } => DocChangeKind::MeshRemoved { id },
        RawHsdChange::MaterialAdded { id } => {
            let data = get_material_at(hsd_map, &id).map(|v| MaterialData::Hsd(Box::new(v)))?;
            DocChangeKind::MaterialAdded { id, data }
        }
        RawHsdChange::MaterialChanged { id } => {
            let data = get_material_at(hsd_map, &id).map(|v| MaterialData::Hsd(Box::new(v)))?;
            DocChangeKind::MaterialChanged { id, data }
        }
        RawHsdChange::MaterialRemoved { id } => DocChangeKind::MaterialRemoved { id },
    };
    Some(DocChange { doc: doc_ent, kind })
}

pub fn apply_doc_changes(
    mut reader: MessageReader<DocChange>,
    docs: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    for change in reader.read() {
        let Ok(registry) = docs.get(change.doc) else {
            continue;
        };
        handle_doc_change(change.doc, &change.kind, &registry.0, &mut commands);
    }
}

#[expect(clippy::too_many_lines)]
fn handle_doc_change(
    doc_ent: Entity,
    kind: &DocChangeKind,
    registry: &Arc<SceneRegistryInner>,
    commands: &mut Commands,
) {
    match kind {
        DocChangeKind::NodeAdded {
            id,
            parent_id,
            data,
        } => {
            let existing = registry
                .node_map
                .lock()
                .expect("node_map lock")
                .get(id)
                .cloned();
            let inner = existing.unwrap_or_else(|| {
                let state = node_state_from_data(data);
                let inner = Arc::new(NodeInner {
                    dirty: false.into(),
                    entity: Mutex::new(None),
                    id: id.clone(),
                    state: Mutex::new(state),
                    tree_id: Mutex::new(None),
                });
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
                inner
            });

            let ent = {
                let existing = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = existing {
                    // entity already spawned (e.g. script DocEventQueue path); just update
                    update_node_components(ent, &inner.id, data, commands);
                    ent
                } else {
                    let node = HsdNode {
                        id: inner.id.clone(),
                        parent_id: parent_id.clone(),
                        data: data.clone(),
                    };
                    let ent = spawn_node_entity(doc_ent, &node, commands);
                    *inner.entity.lock().expect("entity lock") = Some(ent);
                    ent
                }
            };

            if let Some(pid) = parent_id {
                let parent_ent = registry
                    .node_map
                    .lock()
                    .expect("node_map lock")
                    .get(pid)
                    .and_then(|pi| *pi.entity.lock().expect("entity lock"));
                if let Some(parent_ent) = parent_ent {
                    commands.entity(ent).insert(ChildOf(parent_ent));
                }
            }
        }

        DocChangeKind::NodeChanged { id, data } => {
            let inner = registry
                .node_map
                .lock()
                .expect("node_map lock")
                .get(id)
                .cloned();
            if let Some(inner) = inner {
                // Update cached state.
                {
                    let mut state = inner.state.lock().expect("node state lock");
                    state.name = data.name.as_ref().map(std::string::ToString::to_string);
                    state.transform = node_transform(data);
                    state.mesh.clone_from(&data.mesh);
                    state.material.clone_from(&data.material);
                    state.collider.clone_from(&data.collider);
                    state.rigid_body.clone_from(&data.rigid_body);
                    state.scripts = data
                        .scripts
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .map(|h| h.0)
                        .collect();
                }
                let ent = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = ent {
                    update_node_components(ent, &inner.id, data, commands);
                }
            }
        }

        DocChangeKind::NodeParentChanged { id, parent_id } => {
            let ent = registry
                .node_map
                .lock()
                .expect("node_map lock")
                .get(id)
                .and_then(|n| *n.entity.lock().expect("entity lock"));
            let Some(ent) = ent else { return };
            if let Some(pid) = parent_id {
                let parent_ent = registry
                    .node_map
                    .lock()
                    .expect("node_map lock")
                    .get(pid)
                    .and_then(|pi| *pi.entity.lock().expect("entity lock"));
                if let Some(parent_ent) = parent_ent {
                    commands.entity(ent).insert(ChildOf(parent_ent));
                }
            } else {
                commands.entity(ent).remove::<ChildOf>();
            }
        }

        DocChangeKind::NodeRemoved { id } => {
            let removed = registry.node_map.lock().expect("node_map lock").remove(id);
            if let Some(inner) = removed {
                // Remove the canonical gen_id key too if it differs (committed script nodes
                // have both a gen_id and a tree_id alias in node_map).
                if inner.id != *id {
                    registry
                        .node_map
                        .lock()
                        .expect("node_map lock")
                        .remove(&inner.id);
                }
                registry
                    .nodes
                    .lock()
                    .expect("nodes lock")
                    .retain(|n| n.id != inner.id);
                let ent = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = ent {
                    commands.entity(ent).despawn();
                }
            }
        }

        DocChangeKind::MeshAdded { id, data } => {
            let ent = spawn_mesh(doc_ent, data, commands);
            match registry
                .meshes
                .lock()
                .expect("meshes lock")
                .entry(id.clone())
            {
                Entry::Occupied(entry) => {
                    let inner = entry.get();
                    *inner.entity.lock().expect("mesh entity lock") = Some(ent);
                }
                Entry::Vacant(entry) => {
                    let state = match data {
                        MeshData::Hsd(_) => MeshState::default(),
                        MeshData::Inline(state) => state.clone(),
                    };
                    let inner = Arc::new(MeshInner {
                        dirty: false.into(),
                        id: id.clone(),
                        state: Mutex::new(state),
                        entity: Mutex::new(Some(ent)),
                    });
                    entry.insert(inner);
                }
            }
        }

        DocChangeKind::MeshChanged { id, data } => {
            let meshes = registry.meshes.lock().expect("meshes lock");
            if let Some(inner) = meshes.get(id) {
                let ent = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = ent {
                    apply_mesh_data(ent, data, commands);
                }
            }
        }

        DocChangeKind::MeshRemoved { id } => {
            let meshes = registry.meshes.lock().expect("meshes lock");
            if let Some(inner) = meshes.get(id) {
                let ent = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = ent {
                    commands.entity(ent).despawn();
                }
                *inner.entity.lock().expect("entity lock") = None;
            }
        }

        DocChangeKind::MaterialAdded { id, data } => {
            let ent = spawn_material(doc_ent, data, commands);
            match registry
                .materials
                .lock()
                .expect("materials lock")
                .entry(id.clone())
            {
                Entry::Occupied(entry) => {
                    let inner = entry.get();
                    *inner.entity.lock().expect("material entity lock") = Some(ent);
                }
                Entry::Vacant(entry) => {
                    let state = match data {
                        MaterialData::Hsd(_) => MaterialState::default(),
                        MaterialData::Inline(state) => state.clone(),
                    };
                    let inner = Arc::new(MaterialInner {
                        dirty: false.into(),
                        id: id.clone(),
                        state: Mutex::new(state),
                        entity: Mutex::new(Some(ent)),
                    });
                    entry.insert(inner);
                }
            }
        }

        DocChangeKind::MaterialChanged { id, data } => {
            let mats = registry.materials.lock().expect("materials lock");
            if let Some(inner) = mats.get(id) {
                let ent = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = ent {
                    apply_material_data(ent, data, commands);
                }
            }
        }

        DocChangeKind::MaterialRemoved { id } => {
            let mats = registry.materials.lock().expect("materials lock");
            if let Some(inner) = mats.get(id) {
                let ent = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = ent {
                    commands.entity(ent).despawn();
                }
                *inner.entity.lock().expect("entity lock") = None;
            }
        }
    }
}

fn spawn_mesh(doc_ent: Entity, data: &MeshData, commands: &mut Commands) -> Entity {
    match data {
        MeshData::Hsd(hsd_mesh) => commands
            .spawn((HsdChild { doc: doc_ent }, hsd_mesh.clone()))
            .id(),
        MeshData::Inline(state) => {
            let ent = commands.spawn(HsdChild { doc: doc_ent }).id();
            attach_inline_mesh(ent, state, commands);
            ent
        }
    }
}

fn apply_mesh_data(ent: Entity, data: &MeshData, commands: &mut Commands) {
    match data {
        MeshData::Hsd(hsd_mesh) => {
            commands.entity(ent).insert(hsd_mesh.clone());
        }
        MeshData::Inline(state) => {
            commands
                .entity(ent)
                .remove::<BlobDepsLoaded>()
                .remove::<BlobDeps>();
            attach_inline_mesh(ent, state, commands);
        }
    }
}

fn attach_inline_mesh(ent: Entity, state: &MeshState, commands: &mut Commands) {
    let mut attr_deps = Vec::new();

    macro_rules! add_attr {
        ($field:expr, $name:expr) => {
            if let Some(ref data) = $field {
                let bytes = Bytes::copy_from_slice(cast_slice::<f32, u8>(data));
                let dep = commands
                    .spawn((
                        BlobDep { owner: ent },
                        MeshAttrName($name.into()),
                        BlobResponse(Some(bytes)),
                    ))
                    .id();
                attr_deps.push(dep);
            }
        };
    }

    add_attr!(state.positions, "POSITION");
    add_attr!(state.normals, "NORMAL");
    add_attr!(state.tangents, "TANGENT");
    add_attr!(state.colors, "COLOR");
    add_attr!(state.uv0, "UV_0");
    add_attr!(state.uv1, "UV_1");

    let indices = state.indices.as_ref().map(|idx| {
        let bytes = Bytes::copy_from_slice(cast_slice::<u32, u8>(idx));
        commands
            .spawn((BlobDep { owner: ent }, BlobResponse(Some(bytes))))
            .id()
    });

    commands.entity(ent).insert((
        MeshParams {
            topology: state.topology,
            attr_deps,
            indices,
        },
        BlobDepsLoaded,
    ));
}

fn spawn_material(doc_ent: Entity, data: &MaterialData, commands: &mut Commands) -> Entity {
    match data {
        MaterialData::Hsd(hsd_mat) => commands
            .spawn((HsdChild { doc: doc_ent }, hsd_mat.deref().clone()))
            .id(),
        MaterialData::Inline(state) => {
            let ent = commands.spawn(HsdChild { doc: doc_ent }).id();
            attach_inline_material(ent, state, commands);
            ent
        }
    }
}

fn apply_material_data(ent: Entity, data: &MaterialData, commands: &mut Commands) {
    match data {
        MaterialData::Hsd(hsd_mat) => {
            commands.entity(ent).insert(hsd_mat.deref().clone());
        }
        MaterialData::Inline(state) => {
            commands
                .entity(ent)
                .remove::<BlobDepsLoaded>()
                .remove::<BlobDeps>();
            attach_inline_material(ent, state, commands);
        }
    }
}

fn attach_inline_material(ent: Entity, state: &MaterialState, commands: &mut Commands) {
    let [r, g, b, a] = state.base_color;
    commands.entity(ent).insert((
        MaterialParams {
            base_color: Some(Color::srgba(r, g, b, a)),
            double_sided: Some(state.double_sided),
            metallic: Some(state.metallic),
            roughness: Some(state.roughness),
            base_color_texture: None,
            _metallic_roughness_texture: None,
            _normal_texture: None,
            _occlusion_texture: None,
        },
        BlobDepsLoaded,
    ));
}

fn node_state_from_data(data: &HsdNodeData) -> NodeState {
    NodeState {
        name: data.name.as_ref().map(std::string::ToString::to_string),
        transform: node_transform(data),
        mesh: data.mesh.clone(),
        material: data.material.clone(),
        collider: data.collider.clone(),
        rigid_body: data.rigid_body.clone(),
        scripts: data
            .scripts
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|h| h.0)
            .collect(),
        ..Default::default()
    }
}

fn node_data_from_inner(inner: &NodeInner) -> (SmolStr, HsdNodeData) {
    let state = inner.state.lock().expect("node state lock");
    let t = state.transform.translation;
    let r = state.transform.rotation;
    let s = state.transform.scale;
    let data = HsdNodeData {
        name: state.name.clone().map(Into::into),
        translation: Some(vec![f64::from(t.x), f64::from(t.y), f64::from(t.z)]),
        rotation: Some(vec![
            f64::from(r.x),
            f64::from(r.y),
            f64::from(r.z),
            f64::from(r.w),
        ]),
        scale: Some(vec![f64::from(s.x), f64::from(s.y), f64::from(s.z)]),
        mesh: state.mesh.clone(),
        material: state.material.clone(),
        collider: state.collider.clone(),
        rigid_body: state.rigid_body.clone(),
        scripts: if state.scripts.is_empty() {
            None
        } else {
            Some(state.scripts.iter().map(|h| HydratedHash(*h)).collect())
        },
    };
    (inner.id.clone(), data)
}

fn node_data_from_hsd(hsd_map: &LoroMap, tid: TreeID) -> HsdNodeData {
    let tree = hsd_map
        .get_or_create_container("nodes", LoroTree::new())
        .ok();
    tree.as_ref()
        .and_then(|t| t.get_meta(tid).ok())
        .and_then(|m| HsdNodeData::hydrate(&m.get_deep_value()).ok())
        .unwrap_or_default()
}

fn get_mesh_at(hsd_map: &LoroMap, key: &str) -> Option<HsdMesh> {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return None;
    };
    let LoroValue::Map(map) = root.get("meshes")? else {
        return None;
    };
    HsdMesh::hydrate(map.get(key)?).ok()
}

fn get_material_at(hsd_map: &LoroMap, key: &str) -> Option<HsdMaterial> {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return None;
    };
    let LoroValue::Map(map) = root.get("materials")? else {
        return None;
    };
    HsdMaterial::hydrate(map.get(key)?).ok()
}
