use std::collections::hash_map::Entry;
use std::ops::Deref;
use std::sync::{Arc, Mutex, atomic::Ordering};

use bevy::asset::RenderAssetUsages;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use loro::{LoroMap, TreeID};
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
    CompiledMaterial, CompiledMesh, HsdChild, HsdDoc,
    cache::{
        MaterialInner, MaterialState, MeshInner, MeshState, NodeInner, NodeState, SceneRegistry,
        SceneRegistryInner,
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

        // Drain script-event queue.
        if let Some(eq) = event_queue {
            let events: Vec<DocChange> = {
                let mut locked = eq.0.lock().expect("doc event queue lock");
                locked.drain(..).collect()
            };
            for event in events {
                writer.write(event);
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

#[expect(clippy::too_many_arguments)]
pub fn apply_doc_changes(
    mut reader: MessageReader<DocChange>,
    docs: Query<&SceneRegistry>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    compiled_meshes: Query<&CompiledMesh>,
    compiled_mats: Query<&CompiledMaterial>,
) {
    for change in reader.read() {
        let Ok(registry) = docs.get(change.doc) else {
            continue;
        };
        handle_doc_change(
            change.doc,
            &change.kind,
            &registry.0,
            &mut commands,
            &asset_server,
            &mut mesh_assets,
            &mut mat_assets,
            &compiled_meshes,
            &compiled_mats,
        );
    }
}

#[expect(clippy::too_many_arguments, clippy::too_many_lines)]
fn handle_doc_change(
    doc_ent: Entity,
    kind: &DocChangeKind,
    registry: &Arc<SceneRegistryInner>,
    commands: &mut Commands,
    asset_server: &AssetServer,
    mesh_assets: &mut Assets<Mesh>,
    mat_assets: &mut Assets<StandardMaterial>,
    compiled_meshes: &Query<&CompiledMesh>,
    compiled_mats: &Query<&CompiledMaterial>,
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

            let node = HsdNode {
                id: id.clone(),
                parent_id: parent_id.clone(),
                data: data.clone(),
            };
            let ent = spawn_node_entity(doc_ent, &node, commands);
            *inner.entity.lock().expect("entity lock") = Some(ent);

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
                    update_node_components(ent, id, data, commands);
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
            let ent = spawn_mesh(doc_ent, data, commands, asset_server);
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
                    apply_mesh_data(
                        ent,
                        data,
                        commands,
                        asset_server,
                        mesh_assets,
                        compiled_meshes,
                    );
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
            let ent = spawn_material(doc_ent, data, commands, asset_server);
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
                    apply_material_data(
                        ent,
                        data,
                        commands,
                        asset_server,
                        mat_assets,
                        compiled_mats,
                    );
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

fn spawn_mesh(
    doc_ent: Entity,
    data: &MeshData,
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> Entity {
    match data {
        MeshData::Hsd(hsd_mesh) => commands
            .spawn((HsdChild { doc: doc_ent }, hsd_mesh.clone()))
            .id(),
        MeshData::Inline(state) => {
            let mesh = build_mesh_from_state(state);
            let handle = asset_server.add(mesh);
            commands
                .spawn((HsdChild { doc: doc_ent }, CompiledMesh(handle)))
                .id()
        }
    }
}

fn apply_mesh_data(
    ent: Entity,
    data: &MeshData,
    commands: &mut Commands,
    asset_server: &AssetServer,
    mesh_assets: &mut Assets<Mesh>,
    compiled_meshes: &Query<&CompiledMesh>,
) {
    match data {
        MeshData::Hsd(hsd_mesh) => {
            commands.entity(ent).insert(hsd_mesh.clone());
        }
        MeshData::Inline(state) => {
            let mesh = build_mesh_from_state(state);
            if let Ok(CompiledMesh(handle)) = compiled_meshes.get(ent)
                && let Some(asset) = mesh_assets.get_mut(handle)
            {
                *asset = mesh;
            } else {
                let handle = asset_server.add(mesh);
                commands.entity(ent).insert(CompiledMesh(handle));
            }
        }
    }
}

fn spawn_material(
    doc_ent: Entity,
    data: &MaterialData,
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> Entity {
    match data {
        MaterialData::Hsd(hsd_mat) => commands
            .spawn((HsdChild { doc: doc_ent }, hsd_mat.deref().clone()))
            .id(),
        MaterialData::Inline(state) => {
            let mat = build_material_from_state(state);
            let handle = asset_server.add(mat);
            commands
                .spawn((HsdChild { doc: doc_ent }, CompiledMaterial(handle)))
                .id()
        }
    }
}

fn apply_material_data(
    ent: Entity,
    data: &MaterialData,
    commands: &mut Commands,
    asset_server: &AssetServer,
    mat_assets: &mut Assets<StandardMaterial>,
    compiled_mats: &Query<&CompiledMaterial>,
) {
    match data {
        MaterialData::Hsd(hsd_mat) => {
            commands.entity(ent).insert(hsd_mat.deref().clone());
        }
        MaterialData::Inline(state) => {
            let mat = build_material_from_state(state);
            if let Ok(CompiledMaterial(handle)) = compiled_mats.get(ent)
                && let Some(asset) = mat_assets.get_mut(handle)
            {
                *asset = mat;
                return;
            }
            let handle = asset_server.add(mat);
            commands.entity(ent).insert(CompiledMaterial(handle));
        }
    }
}

#[must_use]
pub fn build_mesh_from_state(state: &MeshState) -> Mesh {
    let mut mesh = Mesh::new(state.topology, RenderAssetUsages::all());

    if let Some(indices) = &state.indices {
        mesh.insert_indices(Indices::U32(indices.clone()));
    }
    if let Some(positions) = &state.positions {
        let verts: Vec<[f32; 3]> = positions
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(verts),
        );
    }
    if let Some(normals) = &state.normals {
        let verts: Vec<[f32; 3]> = normals
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(verts),
        );
    }
    if let Some(tangents) = &state.tangents {
        let verts: Vec<[f32; 4]> = tangents
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_TANGENT,
            VertexAttributeValues::Float32x4(verts),
        );
    }
    if let Some(colors) = &state.colors {
        let verts: Vec<[f32; 4]> = colors
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4(verts),
        );
    }
    if let Some(uv0) = &state.uv0 {
        let verts: Vec<[f32; 2]> = uv0.chunks_exact(2).map(|c| [c[0], c[1]]).collect();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(verts),
        );
    }
    if let Some(uv1) = &state.uv1 {
        let verts: Vec<[f32; 2]> = uv1.chunks_exact(2).map(|c| [c[0], c[1]]).collect();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_1,
            VertexAttributeValues::Float32x2(verts),
        );
    }

    mesh
}

#[must_use]
pub fn build_material_from_state(state: &MaterialState) -> StandardMaterial {
    let [r, g, b, a] = state.base_color;
    StandardMaterial {
        base_color: Color::srgba(r, g, b, a),
        metallic: state.metallic,
        perceptual_roughness: state.roughness,
        double_sided: state.double_sided,
        ..default()
    }
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
        .get_or_create_container("nodes", loro::LoroTree::new())
        .ok();
    tree.as_ref()
        .and_then(|t| t.get_meta(tid).ok())
        .and_then(|m| HsdNodeData::hydrate(&m.get_deep_value()).ok())
        .unwrap_or_default()
}

fn get_mesh_at(hsd_map: &LoroMap, key: &str) -> Option<HsdMesh> {
    let value = hsd_map.get_deep_value();
    let loro::LoroValue::Map(root) = &value else {
        return None;
    };
    let loro::LoroValue::Map(map) = root.get("meshes")? else {
        return None;
    };
    HsdMesh::hydrate(map.get(key)?).ok()
}

fn get_material_at(hsd_map: &LoroMap, key: &str) -> Option<HsdMaterial> {
    let value = hsd_map.get_deep_value();
    let loro::LoroValue::Map(root) = &value else {
        return None;
    };
    let loro::LoroValue::Map(map) = root.get("materials")? else {
        return None;
    };
    HsdMaterial::hydrate(map.get(key)?).ok()
}
