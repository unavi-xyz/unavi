use bevy::prelude::*;
use loro::{LoroMap, LoroTree, LoroValue, TreeID, TreeParentId};
use loro_surgeon::Hydrate;
use smol_str::{SmolStr, ToSmolStr};

use crate::{
    HsdDoc,
    cache::{
        MaterialDirty, MaterialHsdChanges, MaterialInner, MaterialState, MeshDirty, MeshHsdChanges,
        MeshInner, MeshState, NodeDirty, NodeHsdChanges, NodeInner, NodeState, SceneRegistry,
    },
    data::{HsdMaterial, HsdMesh, HsdNodeData},
    hydrate::events::{RawChangeQueue, RawHsdChange},
};

use super::compile::material::{
    HsdMaterialAlphaCutoffSet, HsdMaterialAlphaModeSet, HsdMaterialBaseColorSet,
    HsdMaterialDespawned, HsdMaterialDoubleSidedSet, HsdMaterialMetallicSet, HsdMaterialNameSet,
    HsdMaterialRoughnessSet, HsdMaterialSpawned,
};
use super::compile::mesh::{
    HsdMeshDespawned, HsdMeshGeometrySet, HsdMeshSpawned, MeshGeometrySource,
};
use super::compile::node::{
    HsdNodeColliderSet, HsdNodeDespawned, HsdNodeMaterialSet, HsdNodeMeshSet, HsdNodeNameSet,
    HsdNodeParentSet, HsdNodeRigidBodySet, HsdNodeScriptsSet, HsdNodeSpawned, HsdNodeTransformSet,
    node_transform,
};

use std::sync::{Arc, Mutex};

#[expect(clippy::too_many_lines)]
pub(crate) fn process_hsd_queue(
    docs: Query<(Entity, &HsdDoc, &SceneRegistry, &RawChangeQueue)>,
    mut commands: Commands,
) {
    for (doc_ent, hsd_doc, registry, raw_queue) in &docs {
        let raw_changes: Vec<_> = raw_queue
            .0
            .lock()
            .expect("raw queue lock")
            .drain(..)
            .collect();
        if raw_changes.is_empty() {
            continue;
        }

        let hsd_map = hsd_doc.0.get_map("hsd");

        for change in raw_changes {
            match change {
                RawHsdChange::NodeAdded { tree_id, parent_id } => {
                    let id = tree_id.to_smolstr();
                    let data = node_data_from_hsd(&hsd_map, tree_id);
                    let inner = get_or_create_node(registry, &id, tree_id);
                    update_node_state(&inner, &data);

                    if inner.entity.lock().expect("entity lock").is_none() {
                        commands.trigger(HsdNodeSpawned {
                            doc: doc_ent,
                            id: id.clone(),
                        });
                    }

                    let parent = parent_id.map(|pid| pid.to_smolstr());
                    commands.trigger(HsdNodeParentSet {
                        doc: doc_ent,
                        id: id.clone(),
                        parent,
                    });

                    emit_node_fields(doc_ent, &id, &data, &mut commands);
                }

                RawHsdChange::NodeChanged { tree_id } => {
                    let id = tree_id.to_smolstr();
                    let data = node_data_from_hsd(&hsd_map, tree_id);
                    let inner = {
                        registry
                            .0
                            .node_map
                            .lock()
                            .expect("node_map lock")
                            .get(&id)
                            .cloned()
                    };
                    let Some(inner) = inner else { continue };
                    update_node_state(&inner, &data);

                    let parent = get_node_parent(&hsd_map, tree_id);
                    commands.trigger(HsdNodeParentSet {
                        doc: doc_ent,
                        id: id.clone(),
                        parent,
                    });

                    emit_node_fields(doc_ent, &id, &data, &mut commands);
                }

                RawHsdChange::NodeRemoved { tree_id } => {
                    commands.trigger(HsdNodeDespawned {
                        doc: doc_ent,
                        id: tree_id.to_smolstr(),
                    });
                }

                RawHsdChange::MeshAdded { id } => {
                    get_or_create_mesh(registry, &id);
                    commands.trigger(HsdMeshSpawned {
                        doc: doc_ent,
                        id: id.clone(),
                    });
                    if let Some(hsd_mesh) = get_mesh_at(&hsd_map, &id) {
                        commands.trigger(HsdMeshGeometrySet {
                            doc: doc_ent,
                            id,
                            source: MeshGeometrySource::Hsd(Box::new(hsd_mesh)),
                        });
                    }
                }

                RawHsdChange::MeshChanged { id } => {
                    if let Some(hsd_mesh) = get_mesh_at(&hsd_map, &id) {
                        commands.trigger(HsdMeshGeometrySet {
                            doc: doc_ent,
                            id,
                            source: MeshGeometrySource::Hsd(Box::new(hsd_mesh)),
                        });
                    }
                }

                RawHsdChange::MeshRemoved { id } => {
                    commands.trigger(HsdMeshDespawned { doc: doc_ent, id });
                }

                RawHsdChange::MaterialAdded { id } => {
                    get_or_create_material(registry, &id);
                    commands.trigger(HsdMaterialSpawned {
                        doc: doc_ent,
                        id: id.clone(),
                    });
                    if let Some(hsd_mat) = get_material_at(&hsd_map, &id) {
                        emit_material_fields(doc_ent, &id, &hsd_mat, &mut commands);
                    }
                }

                RawHsdChange::MaterialChanged { id } => {
                    if let Some(hsd_mat) = get_material_at(&hsd_map, &id) {
                        emit_material_fields(doc_ent, &id, &hsd_mat, &mut commands);
                    }
                }

                RawHsdChange::MaterialRemoved { id } => {
                    commands.trigger(HsdMaterialDespawned { doc: doc_ent, id });
                }
            }
        }
    }
}

fn get_or_create_node(registry: &SceneRegistry, id: &SmolStr, tree_id: TreeID) -> Arc<NodeInner> {
    let mut node_map = registry.0.node_map.lock().expect("node_map lock");
    if let Some(existing) = node_map.get(id) {
        return Arc::clone(existing);
    }
    let inner = Arc::new(NodeInner {
        dirty: Mutex::new(NodeDirty::default()),
        entity: Mutex::new(None),
        hsd_changes: Mutex::new(NodeHsdChanges::default()),
        id: id.clone(),
        is_virtual: false,
        state: Mutex::new(NodeState::default()),
        sync: false.into(),
        tree_id: Mutex::new(Some(tree_id)),
    });
    node_map.insert(id.clone(), Arc::clone(&inner));
    drop(node_map);
    registry
        .0
        .nodes
        .lock()
        .expect("nodes lock")
        .push(Arc::clone(&inner));
    inner
}

fn get_or_create_mesh(registry: &SceneRegistry, id: &SmolStr) -> Arc<MeshInner> {
    let mut meshes = registry.0.meshes.lock().expect("meshes lock");
    if let Some(existing) = meshes.get(id) {
        return Arc::clone(existing);
    }
    let inner = Arc::new(MeshInner {
        dirty: Mutex::new(MeshDirty::default()),
        entity: Mutex::new(None),
        hsd_changes: Mutex::new(MeshHsdChanges::default()),
        id: id.clone(),
        state: Mutex::new(MeshState::default()),
        sync: false.into(),
    });
    meshes.insert(id.clone(), Arc::clone(&inner));
    inner
}

fn get_or_create_material(registry: &SceneRegistry, id: &SmolStr) -> Arc<MaterialInner> {
    let mut mats = registry.0.materials.lock().expect("materials lock");
    if let Some(existing) = mats.get(id) {
        return Arc::clone(existing);
    }
    let inner = Arc::new(MaterialInner {
        dirty: Mutex::new(MaterialDirty::default()),
        entity: Mutex::new(None),
        hsd_changes: Mutex::new(MaterialHsdChanges::default()),
        id: id.clone(),
        state: Mutex::new(MaterialState::default()),
        sync: false.into(),
    });
    mats.insert(id.clone(), Arc::clone(&inner));
    inner
}

fn update_node_state(inner: &NodeInner, data: &HsdNodeData) {
    let mut state = inner.state.lock().expect("node state lock");
    state.name = data.name.as_ref().map(ToString::to_string);
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

fn emit_node_fields(doc: Entity, id: &SmolStr, data: &HsdNodeData, commands: &mut Commands) {
    commands.trigger(HsdNodeTransformSet {
        doc,
        id: id.clone(),
        transform: node_transform(data),
    });
    commands.trigger(HsdNodeMeshSet {
        doc,
        id: id.clone(),
        mesh: data.mesh.clone(),
    });
    commands.trigger(HsdNodeMaterialSet {
        doc,
        id: id.clone(),
        material: data.material.clone(),
    });
    if let Some(name) = &data.name {
        commands.trigger(HsdNodeNameSet {
            doc,
            id: id.clone(),
            name: Some(name.to_string()),
        });
    }
    commands.trigger(HsdNodeColliderSet {
        doc,
        id: id.clone(),
        collider: data.collider.clone(),
    });
    commands.trigger(HsdNodeRigidBodySet {
        doc,
        id: id.clone(),
        rigid_body: data.rigid_body.clone(),
    });
    let scripts: Vec<blake3::Hash> = data
        .scripts
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|h| h.0)
        .collect();
    commands.trigger(HsdNodeScriptsSet {
        doc,
        id: id.clone(),
        scripts,
    });
}

#[expect(clippy::cast_possible_truncation)]
fn emit_material_fields(doc: Entity, id: &SmolStr, hsd: &HsdMaterial, commands: &mut Commands) {
    if let Some(color) = &hsd.base_color
        && color.len() >= 3
    {
        let r = color[0] as f32;
        let g = color[1] as f32;
        let b = color[2] as f32;
        let a = color.get(3).copied().unwrap_or(1.0) as f32;
        commands.trigger(HsdMaterialBaseColorSet {
            doc,
            id: id.clone(),
            color: [r, g, b, a],
        });
    }
    if let Some(v) = hsd.metallic {
        commands.trigger(HsdMaterialMetallicSet {
            doc,
            id: id.clone(),
            value: v as f32,
        });
    }
    if let Some(v) = hsd.roughness {
        commands.trigger(HsdMaterialRoughnessSet {
            doc,
            id: id.clone(),
            value: v as f32,
        });
    }
    if let Some(v) = hsd.alpha_cutoff {
        commands.trigger(HsdMaterialAlphaCutoffSet {
            doc,
            id: id.clone(),
            value: v as f32,
        });
    }
    if let Some(ref mode) = hsd.alpha_mode {
        commands.trigger(HsdMaterialAlphaModeSet {
            doc,
            id: id.clone(),
            mode: Some(mode.to_string()),
        });
    }
    if let Some(v) = hsd.double_sided {
        commands.trigger(HsdMaterialDoubleSidedSet {
            doc,
            id: id.clone(),
            value: v,
        });
    }
    if let Some(ref name) = hsd.name {
        commands.trigger(HsdMaterialNameSet {
            doc,
            id: id.clone(),
            name: Some(name.to_string()),
        });
    }
}

fn get_node_parent(hsd_map: &LoroMap, tree_id: TreeID) -> Option<SmolStr> {
    let tree = hsd_map
        .get_or_create_container("nodes", LoroTree::new())
        .ok()?;
    match tree.parent(tree_id)? {
        TreeParentId::Node(pid) => Some(pid.to_smolstr()),
        _ => None,
    }
}

pub(super) fn node_data_from_hsd(hsd_map: &LoroMap, tid: TreeID) -> HsdNodeData {
    let tree = hsd_map
        .get_or_create_container("nodes", LoroTree::new())
        .ok();
    tree.as_ref()
        .and_then(|t| t.get_meta(tid).ok())
        .and_then(|m| HsdNodeData::hydrate(&m.get_deep_value()).ok())
        .unwrap_or_default()
}

pub(super) fn get_mesh_at(hsd_map: &LoroMap, key: &str) -> Option<HsdMesh> {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return None;
    };
    let LoroValue::Map(map) = root.get("meshes")? else {
        return None;
    };
    HsdMesh::hydrate(map.get(key)?).ok()
}

pub(super) fn get_material_at(hsd_map: &LoroMap, key: &str) -> Option<HsdMaterial> {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return None;
    };
    let LoroValue::Map(map) = root.get("materials")? else {
        return None;
    };
    HsdMaterial::hydrate(map.get(key)?).ok()
}
