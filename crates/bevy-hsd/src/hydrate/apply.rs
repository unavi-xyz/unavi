use std::collections::hash_map::Entry;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobResponse};
use bytemuck::cast_slice;
use bytes::Bytes;
use loro::{LoroMap, LoroTree, LoroValue, TreeID};
use loro_surgeon::Hydrate;
use smol_str::{SmolStr, ToSmolStr};
use wired_schemas::HydratedHash;

use super::{
    events::{DocChange, DocChangeKind, DocChangeQueue, MaterialData, MeshData, RawHsdChange},
    node::{node_transform, spawn_node_entity, update_node_components},
};
use crate::{
    HsdChild, MaterialRef, MeshRef,
    cache::{
        MaterialChanges, MaterialInner, MaterialState, MeshChanges, MeshInner, MeshState,
        NodeChanges, NodeInner, NodeState, SceneRegistry, SceneRegistryInner,
    },
    compile::{
        material::MaterialParams,
        mesh::{MeshAttrName, MeshParams},
    },
    data::{HsdCollider, HsdMaterial, HsdMesh, HsdNode, HsdNodeData, HsdRigidBody},
};

pub(crate) fn flush_scene_dirty(docs: Query<(Entity, &SceneRegistry, &DocChangeQueue)>) {
    for (doc_ent, registry, queue) in &docs {
        let mut changes = queue.0.lock().expect("change queue lock");
        {
            let nodes = registry.0.nodes.lock().expect("nodes lock");
            for inner in nodes.iter() {
                if inner.is_virtual {
                    continue;
                }
                let node_changes = {
                    let mut ch = inner.changes.lock().expect("changes lock");
                    if ch.is_empty() {
                        continue;
                    }
                    std::mem::take(&mut *ch)
                };
                let has_entity = inner.entity.lock().expect("entity lock").is_some();
                let kind = if has_entity {
                    DocChangeKind::NodeScriptChanged {
                        id: inner.id.clone(),
                        changes: Box::new(node_changes),
                    }
                } else {
                    let parent_id = inner
                        .state
                        .lock()
                        .expect("node state lock")
                        .parent
                        .as_ref()
                        .and_then(std::sync::Weak::upgrade)
                        .map(|pi| pi.id.clone());
                    let (id, data) = node_data_from_inner(inner);
                    DocChangeKind::NodeAdded {
                        id,
                        parent_id,
                        data,
                    }
                };
                changes.push(DocChange { doc: doc_ent, kind });
            }
        }
        {
            let meshes = registry.0.meshes.lock().expect("meshes lock");
            for (id, inner) in meshes.iter() {
                debug_assert_eq!(*id, inner.id);
                let (mesh_changes, state) = {
                    let mut ch = inner.changes.lock().expect("changes lock");
                    if ch.is_empty() {
                        continue;
                    }
                    let mesh_changes = std::mem::take(&mut *ch);
                    drop(ch);
                    let state = inner.state.lock().expect("mesh state lock").clone();
                    (mesh_changes, state)
                };
                changes.push(DocChange {
                    doc: doc_ent,
                    kind: DocChangeKind::MeshScriptChanged {
                        id: id.clone(),
                        changes: Box::new(mesh_changes),
                        state: Box::new(state),
                    },
                });
            }
        }
        {
            let materials = registry.0.materials.lock().expect("materials lock");
            for (id, inner) in materials.iter() {
                debug_assert_eq!(*id, inner.id);
                let (mat_changes, state) = {
                    let mut ch = inner.changes.lock().expect("changes lock");
                    if ch.is_empty() {
                        continue;
                    }
                    let mat_changes = std::mem::take(&mut *ch);
                    drop(ch);
                    let state = inner.state.lock().expect("material state lock").clone();
                    (mat_changes, state)
                };
                changes.push(DocChange {
                    doc: doc_ent,
                    kind: DocChangeKind::MaterialScriptChanged {
                        id: id.clone(),
                        changes: Box::new(mat_changes),
                        state: Box::new(state),
                    },
                });
            }
        }
    }
}

pub(super) fn raw_to_doc_change(
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

pub(crate) fn apply_doc_changes(
    docs: Query<(&SceneRegistry, &DocChangeQueue)>,
    mut commands: Commands,
) {
    for (registry, queue) in &docs {
        let changes: Vec<DocChange> = queue
            .0
            .lock()
            .expect("change queue lock")
            .drain(..)
            .collect();
        for change in changes {
            handle_doc_change(change.doc, &change.kind, &registry.0, &mut commands);
        }
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
                    changes: Mutex::new(NodeChanges::default()),
                    entity: Mutex::new(None),
                    id: id.clone(),
                    is_virtual: false,
                    state: Mutex::new(state),
                    sync: false.into(),
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

        DocChangeKind::NodeScriptChanged { id, changes } => {
            let inner = registry
                .node_map
                .lock()
                .expect("node_map lock")
                .get(id)
                .cloned();
            let Some(inner) = inner else { return };
            let ent = *inner.entity.lock().expect("entity lock");
            let Some(ent) = ent else { return };
            let mut ecmd = commands.entity(ent);

            if changes.translation.is_some()
                || changes.rotation.is_some()
                || changes.scale.is_some()
            {
                let state = inner.state.lock().expect("node state lock");
                ecmd.insert(state.transform);
            }
            if let Some(new_mesh) = &changes.mesh {
                ecmd.remove::<MeshRef>();
                if let Some(id) = new_mesh {
                    ecmd.insert(MeshRef(id.clone()));
                }
            }
            if let Some(new_mat) = &changes.material {
                ecmd.remove::<MaterialRef>();
                if let Some(id) = new_mat {
                    ecmd.insert(MaterialRef(id.clone()));
                }
            }
            if let Some(new_collider) = &changes.collider {
                ecmd.remove::<HsdCollider>();
                if let Some(c) = new_collider {
                    ecmd.insert(c.clone());
                }
            }
            if let Some(new_rb) = &changes.rigid_body {
                ecmd.remove::<HsdRigidBody>();
                if let Some(rb) = new_rb {
                    ecmd.insert(rb.clone());
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
                        changes: Mutex::new(MeshChanges::default()),
                        entity: Mutex::new(Some(ent)),
                        id: id.clone(),
                        state: Mutex::new(state),
                        sync: false.into(),
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
                } else {
                    warn!("MeshChanged: no entity");
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

        DocChangeKind::MeshScriptChanged { id, changes, state } => {
            let meshes = registry.meshes.lock().expect("meshes lock");
            if let Some(inner) = meshes.get(id) {
                let ent = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = ent
                    && changes.geometry {
                        commands
                            .entity(ent)
                            .remove::<BlobDepsLoaded>()
                            .remove::<BlobDeps>();
                        attach_inline_mesh(ent, state, commands);
                    }
                    // topology/name-only changes don't have separate Bevy components
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
                        changes: Mutex::new(MaterialChanges::default()),
                        entity: Mutex::new(Some(ent)),
                        id: id.clone(),
                        state: Mutex::new(state),
                        sync: false.into(),
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
                } else {
                    warn!("MaterialChanged: no entity");
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

        DocChangeKind::MaterialScriptChanged { id, changes: _, state } => {
            let mats = registry.materials.lock().expect("materials lock");
            if let Some(inner) = mats.get(id) {
                let ent = *inner.entity.lock().expect("entity lock");
                if let Some(ent) = ent {
                    commands
                        .entity(ent)
                        .remove::<BlobDepsLoaded>()
                        .remove::<BlobDeps>();
                    attach_inline_material(ent, state, commands);
                }
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
    commands
        .entity(ent)
        .remove::<BlobDepsLoaded>()
        .remove::<BlobDeps>();

    match data {
        MeshData::Hsd(hsd_mesh) => {
            commands.entity(ent).insert(hsd_mesh.clone());
        }
        MeshData::Inline(state) => {
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

    commands.entity(ent).insert(MeshParams {
        topology: state.topology,
        attr_deps,
        indices,
    });
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
    commands
        .entity(ent)
        .remove::<BlobDepsLoaded>()
        .remove::<BlobDeps>();

    match data {
        MaterialData::Hsd(hsd_mat) => {
            commands.entity(ent).insert(hsd_mat.deref().clone());
        }
        MaterialData::Inline(state) => {
            attach_inline_material(ent, state, commands);
        }
    }
}

fn attach_inline_material(ent: Entity, state: &MaterialState, commands: &mut Commands) {
    let [r, g, b, a] = state.base_color;
    commands.entity(ent).insert(MaterialParams {
        alpha_cutoff: state.alpha_cutoff,
        alpha_mode: state.alpha_mode.clone(),
        base_color: Some(Color::srgba(r, g, b, a)),
        double_sided: Some(state.double_sided),
        metallic: Some(state.metallic),
        roughness: Some(state.roughness),
        base_color_texture: None,
        _metallic_roughness_texture: None,
        _normal_texture: None,
        _occlusion_texture: None,
    });
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
