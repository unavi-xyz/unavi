use std::sync::{Arc, Mutex};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use loro::{LoroMap, LoroValue};
use loro_surgeon::Hydrate;
use smol_str::SmolStr;

use super::diff::extract_changes_from_diff;
use super::node::{spawn_node_entity, update_node_components};
use crate::{
    HsdChange, HsdChild, HsdDoc, HsdNodeTreeId, HsdScripts, HsdSubscription, MaterialRef, MeshRef,
    NodeId,
    cache::{
        MaterialInner, MaterialState, MeshInner, MeshState, NodeInner, NodeState, SceneEvent,
        SceneEventQueue, SceneRegistry, SceneRegistryInner,
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
        let events: Arc<Mutex<Vec<SceneEvent>>> = Arc::new(Mutex::new(Vec::new()));

        let hsd_map = doc.get_map("hsd");
        full_hydrate(&hsd_map, doc_ent, &mut commands, &registry);

        // Subscribe: future doc changes → HsdChange queue → processed below.
        let change_queue: Arc<Mutex<Vec<HsdChange>>> =
            Arc::new(Mutex::new(Vec::<HsdChange>::new()));
        let cq = Arc::clone(&change_queue);
        let sub = doc.subscribe_root(Arc::new(move |e| {
            if let Ok(mut locked) = cq.try_lock() {
                extract_changes_from_diff(&e, &mut locked);
            }
        }));

        // Store change queue alongside registry/events so apply_scene_events
        // can pick up Loro-subscription events.
        let change_queue_comp = HsdChangeQueue(change_queue);

        commands.entity(doc_ent).insert((
            SceneRegistry(Arc::clone(&registry)),
            SceneEventQueue(Arc::clone(&events)),
            change_queue_comp,
            HsdSubscription(sub),
        ));
    }
}

#[derive(Component, Clone)]
pub struct HsdChangeQueue(pub Arc<Mutex<Vec<HsdChange>>>);

pub(crate) fn apply_scene_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut docs: Query<(
        Entity,
        &HsdDoc,
        &SceneRegistry,
        &SceneEventQueue,
        Option<&HsdChangeQueue>,
    )>,
) {
    for (doc_ent, hsd_doc, registry, event_queue, change_queue) in &mut docs {
        if let Some(cq) = change_queue {
            let changes: Vec<HsdChange> = {
                let mut locked = cq.0.lock().expect("change queue lock");
                locked.drain(..).collect()
            };
            let hsd_map = hsd_doc.0.get_map("hsd");
            apply_hsd_changes(doc_ent, &changes, &hsd_map, &registry.0, &mut commands);
        }

        let events: Vec<SceneEvent> = {
            let mut locked = event_queue.0.lock().expect("event queue lock");
            locked.drain(..).collect()
        };

        for event in events {
            apply_scene_event(doc_ent, event, &mut commands, &asset_server);
        }
    }
}

pub(crate) fn sync_ecs_to_cache(
    registries: Query<&SceneRegistry>,
    transforms: Query<&GlobalTransform>,
) {
    for registry in &registries {
        let nodes = registry.0.nodes.lock().expect("nodes lock");
        for node_inner in nodes.iter() {
            let ent = *node_inner.entity.lock().expect("entity lock");
            let Some(ent) = ent else { continue };
            let Ok(gt) = transforms.get(ent) else {
                continue;
            };
            let mut state = node_inner.state.lock().expect("node state lock");
            state.global_transform = *gt;
        }
    }
}

#[expect(clippy::too_many_lines)]
fn apply_hsd_changes(
    doc_ent: Entity,
    changes: &[HsdChange],
    hsd_map: &LoroMap,
    registry: &Arc<SceneRegistryInner>,
    commands: &mut Commands,
) {
    for change in changes {
        match change {
            HsdChange::NodeAdded { tree_id, parent_id } => {
                // Check if already in registry (locally created).
                let existing = loro::TreeID::try_from(tree_id.as_str()).map_or_else(
                    |_| None,
                    |tid| {
                        registry
                            .node_map
                            .lock()
                            .expect("node_map lock")
                            .get(&tid)
                            .cloned()
                    },
                );
                if existing.is_none() {
                    // New node from network.
                    let Ok(tid) = loro::TreeID::try_from(tree_id.as_str()) else {
                        continue;
                    };
                    let state = node_state_from_hsd(tid, hsd_map);
                    let inner = Arc::new(NodeInner {
                        tree_id: tid,
                        state: Mutex::new(state),
                        entity: Mutex::new(None),
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
                        .insert(tid, Arc::clone(&inner));
                    // Spawn entity directly.
                    let ent = spawn_node_entity_from_inner(doc_ent, &inner, commands);
                    *inner.entity.lock().expect("entity lock") = Some(ent);

                    // Parent.
                    if let Some(pid) = parent_id
                        && let Ok(parent_tid) = loro::TreeID::try_from(pid.as_str())
                    {
                        let parent_inner = {
                            registry
                                .node_map
                                .lock()
                                .expect("node_map lock")
                                .get(&parent_tid)
                                .cloned()
                        };
                        if let Some(pi) = parent_inner {
                            let parent_ent = *pi.entity.lock().expect("entity lock");
                            if let Some(parent_ent) = parent_ent {
                                commands.entity(ent).insert(ChildOf(parent_ent));
                            }
                        }
                    }
                }
            }

            HsdChange::NodeRemoved { tree_id } => {
                let Ok(tid) = loro::TreeID::try_from(tree_id.as_str()) else {
                    continue;
                };
                let removed = {
                    let mut node_map = registry.node_map.lock().expect("node_map lock");
                    node_map.remove(&tid)
                };
                if let Some(inner) = removed {
                    let mut nodes = registry.nodes.lock().expect("nodes lock");
                    nodes.retain(|n| n.tree_id != inner.tree_id);
                    drop(nodes);
                    let ent = *inner.entity.lock().expect("entity lock");
                    if let Some(ent) = ent {
                        commands.entity(ent).despawn();
                    }
                }
            }

            HsdChange::NodeMetaChanged { tree_id } => {
                let Ok(tid) = loro::TreeID::try_from(tree_id.as_str()) else {
                    continue;
                };
                let inner = registry
                    .node_map
                    .lock()
                    .expect("node_map lock")
                    .get(&tid)
                    .cloned();
                if let Some(inner) = inner {
                    // Update NodeState from Loro.
                    let new_state = node_state_from_hsd(tid, hsd_map);
                    {
                        let mut state = inner.state.lock().expect("node state lock");
                        state.name = new_state.name;
                        state.transform = new_state.transform;
                        state.mesh = new_state.mesh;
                        state.material = new_state.material;
                        state.collider = new_state.collider;
                        state.rigid_body = new_state.rigid_body;
                        state.scripts = new_state.scripts;
                    }
                    // Update entity.
                    let ent = *inner.entity.lock().expect("entity lock");
                    if let Some(ent) = ent {
                        update_node_components(ent, tree_id.as_str(), hsd_map, commands);
                    }
                }
            }

            HsdChange::MeshAdded => {
                let index = {
                    let meshes = registry.meshes.lock().expect("meshes lock");
                    meshes.len()
                };
                let mesh = get_mesh_at(hsd_map, index);
                if let Some(mesh_data) = mesh {
                    let ent = commands.spawn((HsdChild { doc: doc_ent }, mesh_data)).id();
                    let inner = Arc::new(MeshInner {
                        index,
                        state: Mutex::new(MeshState::default()),
                        entity: Mutex::new(Some(ent)),
                    });
                    registry.meshes.lock().expect("meshes lock").push(inner);
                } else {
                    let inner = Arc::new(MeshInner {
                        index,
                        state: Mutex::new(MeshState::default()),
                        entity: Mutex::new(None),
                    });
                    registry.meshes.lock().expect("meshes lock").push(inner);
                }
            }

            HsdChange::MeshRemoved { index } => {
                let meshes = registry.meshes.lock().expect("meshes lock");
                if let Some(inner) = meshes.get(*index) {
                    let ent = *inner.entity.lock().expect("entity lock");
                    if let Some(ent) = ent {
                        commands.entity(ent).despawn();
                    }
                    // Don't remove from vec to preserve indices; just clear entity.
                    if let Some(inner) = meshes.get(*index) {
                        *inner.entity.lock().expect("entity lock") = None;
                    }
                }
                drop(meshes);
            }

            HsdChange::MeshChanged { index } => {
                // Despawn old entity, spawn new.
                let old_ent = {
                    let meshes = registry.meshes.lock().expect("meshes lock");
                    meshes
                        .get(*index)
                        .and_then(|i| *i.entity.lock().expect("entity lock"))
                };
                if let Some(ent) = old_ent {
                    commands.entity(ent).despawn();
                }
                if let Some(mesh_data) = get_mesh_at(hsd_map, *index) {
                    let ent = commands.spawn((HsdChild { doc: doc_ent }, mesh_data)).id();
                    let meshes = registry.meshes.lock().expect("meshes lock");
                    if let Some(inner) = meshes.get(*index) {
                        *inner.entity.lock().expect("entity lock") = Some(ent);
                    }
                }
            }

            HsdChange::MaterialAdded => {
                let index = {
                    let mats = registry.materials.lock().expect("materials lock");
                    mats.len()
                };
                let mat = get_material_at(hsd_map, index);
                if let Some(mat_data) = mat {
                    let ent = commands.spawn((HsdChild { doc: doc_ent }, mat_data)).id();
                    let inner = Arc::new(MaterialInner {
                        index,
                        state: Mutex::new(MaterialState::default()),
                        entity: Mutex::new(Some(ent)),
                    });
                    registry
                        .materials
                        .lock()
                        .expect("materials lock")
                        .push(inner);
                } else {
                    let inner = Arc::new(MaterialInner {
                        index,
                        state: Mutex::new(MaterialState::default()),
                        entity: Mutex::new(None),
                    });
                    registry
                        .materials
                        .lock()
                        .expect("materials lock")
                        .push(inner);
                }
            }

            HsdChange::MaterialRemoved { index } => {
                let mats = registry.materials.lock().expect("materials lock");
                if let Some(inner) = mats.get(*index) {
                    let ent = *inner.entity.lock().expect("entity lock");
                    if let Some(ent) = ent {
                        commands.entity(ent).despawn();
                    }
                    *inner.entity.lock().expect("entity lock") = None;
                }
                drop(mats);
            }

            HsdChange::MaterialChanged { index } => {
                let old_ent = {
                    let mats = registry.materials.lock().expect("materials lock");
                    mats.get(*index)
                        .and_then(|i| *i.entity.lock().expect("entity lock"))
                };
                if let Some(ent) = old_ent {
                    commands.entity(ent).despawn();
                }
                if let Some(mat_data) = get_material_at(hsd_map, *index) {
                    let ent = commands.spawn((HsdChild { doc: doc_ent }, mat_data)).id();
                    let mats = registry.materials.lock().expect("materials lock");
                    if let Some(inner) = mats.get(*index) {
                        *inner.entity.lock().expect("entity lock") = Some(ent);
                    }
                }
            }
        }
    }
}

#[expect(clippy::too_many_lines)]
fn apply_scene_event(
    doc_ent: Entity,
    event: SceneEvent,
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    match event {
        SceneEvent::NodeCreated(inner) => {
            let state = inner.state.lock().expect("node state lock");
            let tree_id_str: SmolStr =
                format!("{}@{}", inner.tree_id.counter, inner.tree_id.peer).into();
            let name: SmolStr = state
                .name
                .clone()
                .map_or_else(|| tree_id_str.clone(), SmolStr::from);

            let mut ent_cmd = commands.spawn((
                HsdChild { doc: doc_ent },
                HsdNodeTreeId(tree_id_str),
                NodeId(name),
                state.transform,
            ));
            if let Some(idx) = state.mesh {
                ent_cmd.insert(MeshRef(idx));
            }
            if let Some(idx) = state.material {
                ent_cmd.insert(MaterialRef(idx));
            }
            if let Some(c) = &state.collider {
                ent_cmd.insert(c.clone());
            }
            if let Some(rb) = &state.rigid_body {
                ent_cmd.insert(rb.clone());
            }
            if !state.scripts.is_empty() {
                ent_cmd.insert(HsdScripts(state.scripts.clone()));
            }
            let ent = ent_cmd.id();
            drop(state);
            *inner.entity.lock().expect("entity lock") = Some(ent);
        }

        SceneEvent::NodeDirty(inner) => {
            let ent = *inner.entity.lock().expect("entity lock");
            let Some(ent) = ent else { return };
            let state = inner.state.lock().expect("node state lock");
            let tree_id_str: SmolStr =
                format!("{}@{}", inner.tree_id.counter, inner.tree_id.peer).into();
            let name: SmolStr = state
                .name
                .clone()
                .map_or_else(|| tree_id_str, SmolStr::from);

            let mut ecmd = commands.entity(ent);
            ecmd.insert((NodeId(name), state.transform));
            ecmd.remove::<(
                Mesh3d,
                MeshMaterial3d<StandardMaterial>,
                MeshRef,
                MaterialRef,
            )>();
            if let Some(idx) = state.mesh {
                ecmd.insert(MeshRef(idx));
            }
            if let Some(idx) = state.material {
                ecmd.insert(MaterialRef(idx));
            }
            let hashes: Vec<blake3::Hash> = state.scripts.clone();
            drop(state);
            if hashes.is_empty() {
                ecmd.remove::<HsdScripts>();
            } else {
                ecmd.insert(HsdScripts(hashes));
            }
        }

        SceneEvent::NodeParentChanged { node, parent } => {
            let ent = *node.entity.lock().expect("entity lock");
            let Some(ent) = ent else { return };
            if let Some(parent_inner) = parent {
                let parent_ent = *parent_inner.entity.lock().expect("entity lock");
                if let Some(parent_ent) = parent_ent {
                    commands.entity(ent).insert(ChildOf(parent_ent));
                }
            } else {
                commands.entity(ent).remove::<ChildOf>();
            }
        }

        SceneEvent::NodeDestroyed(inner) => {
            let ent = *inner.entity.lock().expect("entity lock");
            if let Some(ent) = ent {
                commands.entity(ent).despawn();
            }
        }

        SceneEvent::MeshCreated(inner) => {
            let state = inner.state.lock().expect("mesh state lock");
            let mesh = build_mesh_from_state(&state);
            drop(state);
            let handle = asset_server.add(mesh);
            let ent = commands
                .spawn((HsdChild { doc: doc_ent }, crate::CompiledMesh(handle)))
                .id();
            *inner.entity.lock().expect("entity lock") = Some(ent);
        }

        SceneEvent::MeshDirty(inner) => {
            let ent = *inner.entity.lock().expect("entity lock");
            let Some(ent) = ent else { return };
            let state = inner.state.lock().expect("mesh state lock");
            let mesh = build_mesh_from_state(&state);
            drop(state);
            let handle = asset_server.add(mesh);
            commands.entity(ent).insert(crate::CompiledMesh(handle));
        }

        SceneEvent::MaterialCreated(inner) => {
            let state = inner.state.lock().expect("material state lock");
            let mat = build_material_from_state(&state);
            drop(state);
            let handle = asset_server.add(mat);
            let ent = commands
                .spawn((HsdChild { doc: doc_ent }, crate::CompiledMaterial(handle)))
                .id();
            *inner.entity.lock().expect("entity lock") = Some(ent);
        }

        SceneEvent::MaterialDirty(inner) => {
            let ent = *inner.entity.lock().expect("entity lock");
            let Some(ent) = ent else { return };
            let state = inner.state.lock().expect("material state lock");
            let mat = build_material_from_state(&state);
            drop(state);
            let handle = asset_server.add(mat);
            commands.entity(ent).insert(crate::CompiledMaterial(handle));
        }
    }
}

fn build_mesh_from_state(state: &MeshState) -> Mesh {
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

fn build_material_from_state(state: &MaterialState) -> StandardMaterial {
    let [r, g, b, a] = state.base_color;
    StandardMaterial {
        base_color: Color::srgba(r, g, b, a),
        metallic: state.metallic,
        perceptual_roughness: state.roughness,
        double_sided: state.double_sided,
        ..default()
    }
}

fn full_hydrate(
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

    // Materials.
    for (index, mat) in hsd_data.materials.iter().enumerate() {
        let ent = commands
            .spawn((HsdChild { doc: doc_ent }, HsdMaterial::clone(mat)))
            .id();
        let inner = Arc::new(MaterialInner {
            index,
            state: Mutex::new(MaterialState::default()),
            entity: Mutex::new(Some(ent)),
        });
        registry
            .materials
            .lock()
            .expect("materials lock")
            .push(inner);
    }

    // Meshes.
    for (index, mesh) in hsd_data.meshes.iter().enumerate() {
        let ent = commands
            .spawn((HsdChild { doc: doc_ent }, HsdMesh::clone(mesh)))
            .id();
        let inner = Arc::new(MeshInner {
            index,
            state: Mutex::new(MeshState::default()),
            entity: Mutex::new(Some(ent)),
        });
        registry.meshes.lock().expect("meshes lock").push(inner);
    }

    // Nodes — two passes: spawn then parent.
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
            transform: node_transform_from_data(&node.data),
            mesh: node.data.mesh.and_then(|i| usize::try_from(i).ok()),
            material: node.data.material.and_then(|i| usize::try_from(i).ok()),
            ..Default::default()
        };
        let inner = Arc::new(NodeInner {
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

    // Parent-child relationships.
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

fn node_state_from_hsd(tid: loro::TreeID, hsd_map: &LoroMap) -> NodeState {
    let tree = hsd_map
        .get_or_create_container("nodes", loro::LoroTree::new())
        .ok();
    let data = tree
        .as_ref()
        .and_then(|t| t.get_meta(tid).ok())
        .and_then(|m| {
            let v = m.get_deep_value();
            crate::data::HsdNodeData::hydrate(&v).ok()
        });

    let data = data.unwrap_or_default();
    NodeState {
        name: data.name.as_ref().map(std::string::ToString::to_string),
        transform: node_transform_from_data(&data),
        mesh: data.mesh.and_then(|i| usize::try_from(i).ok()),
        material: data.material.and_then(|i| usize::try_from(i).ok()),
        ..Default::default()
    }
}

fn node_transform_from_data(data: &crate::data::HsdNodeData) -> Transform {
    super::node::node_transform(data)
}

fn spawn_node_entity_from_inner(
    doc_ent: Entity,
    inner: &Arc<NodeInner>,
    commands: &mut Commands,
) -> Entity {
    let state = inner.state.lock().expect("node state lock");
    let tree_id_str: SmolStr = format!("{}@{}", inner.tree_id.counter, inner.tree_id.peer).into();
    let name: SmolStr = state
        .name
        .clone()
        .map_or_else(|| tree_id_str.clone(), SmolStr::from);

    let mut ent_cmd = commands.spawn((
        HsdChild { doc: doc_ent },
        HsdNodeTreeId(tree_id_str),
        NodeId(name),
        state.transform,
    ));
    if let Some(idx) = state.mesh {
        ent_cmd.insert(MeshRef(idx));
    }
    if let Some(idx) = state.material {
        ent_cmd.insert(MaterialRef(idx));
    }
    drop(state);
    ent_cmd.id()
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
