use bevy::prelude::*;
use hsd::{HsdImage, HsdMaterial, HsdMesh, HsdNode};
use loro::{LoroMap, LoroTree, LoroValue, TreeID, TreeParentId};
use loro_surgeon::Hydrate;
use smol_str::{SmolStr, ToSmolStr};

use super::compile::{
    image::{HsdImageDespawned, HsdImageSpawned},
    material::{
        HsdMaterialAlphaCutoffSet, HsdMaterialAlphaModeSet, HsdMaterialBaseColorSet,
        HsdMaterialBaseColorTextureSet, HsdMaterialDespawned, HsdMaterialDoubleSidedSet,
        HsdMaterialEmissiveTextureSet, HsdMaterialMetallicRoughnessTextureSet,
        HsdMaterialMetallicSet, HsdMaterialNameSet, HsdMaterialNormalTextureSet,
        HsdMaterialOcclusionTextureSet, HsdMaterialRoughnessSet, HsdMaterialSpawned,
        HsdMaterialUnlitSet,
    },
    mesh::{HsdMeshDespawned, HsdMeshGeometrySet, HsdMeshSpawned, MeshGeometrySource},
    node::{
        HsdNodeColliderSet, HsdNodeDespawned, HsdNodeMaterialSet, HsdNodeMeshSet, HsdNodeNameSet,
        HsdNodeParentSet, HsdNodeRigidBodySet, HsdNodeScriptsSet, HsdNodeSpawned,
        HsdNodeTransformSet, node_transform,
    },
};

use crate::{
    HsdDoc, HsdRecordId,
    hydrate::events::{NodeRef, RawChangeQueue, RawHsdChange},
};

pub fn process_hsd_queue(
    docs: Query<(&HsdRecordId, &HsdDoc, &RawChangeQueue)>,
    mut commands: Commands,
) {
    for (record_id, hsd_doc, raw_queue) in &docs {
        let doc_id = record_id.0;
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
                RawHsdChange::ImageAdded { id } => {
                    let initial = get_image_at(&hsd_map, &id);
                    commands.trigger(HsdImageSpawned {
                        doc_id,
                        id,
                        initial,
                    });
                }
                RawHsdChange::ImageChanged { id } => {
                    if let Some(hsd_img) = get_image_at(&hsd_map, &id) {
                        commands.trigger(HsdImageSpawned {
                            doc_id,
                            id,
                            initial: Some(hsd_img),
                        });
                    }
                }
                RawHsdChange::ImageRemoved { id } => {
                    commands.trigger(HsdImageDespawned { doc_id, id });
                }
                RawHsdChange::NodeAdded { tree_id, parent_id } => {
                    let id = tree_id.to_smolstr();
                    let data = node_data_from_hsd(&hsd_map, tree_id);

                    commands.trigger(HsdNodeSpawned {
                        doc_id,
                        id: id.clone(),
                    });

                    let parent = parent_id.map(|pid| NodeRef::Id(pid.to_smolstr()));
                    commands.trigger(HsdNodeParentSet {
                        doc_id,
                        child: NodeRef::Id(id.clone()),
                        parent,
                    });

                    emit_node_fields(doc_id, &id, &data, &mut commands);
                }
                RawHsdChange::NodeChanged { tree_id } => {
                    let id = tree_id.to_smolstr();
                    let data = node_data_from_hsd(&hsd_map, tree_id);

                    let parent = get_node_parent(&hsd_map, tree_id).map(NodeRef::Id);
                    commands.trigger(HsdNodeParentSet {
                        doc_id,
                        child: NodeRef::Id(id.clone()),
                        parent,
                    });

                    emit_node_fields(doc_id, &id, &data, &mut commands);
                }
                RawHsdChange::NodeRemoved { tree_id } => {
                    commands.trigger(HsdNodeDespawned {
                        doc_id,
                        id: tree_id.to_smolstr(),
                    });
                }
                RawHsdChange::MeshAdded { id } => {
                    commands.trigger(HsdMeshSpawned {
                        doc_id,
                        id: id.clone(),
                    });
                    if let Some(hsd_mesh) = get_mesh_at(&hsd_map, &id)
                        && (!hsd_mesh.attributes.is_empty() || hsd_mesh.indices.is_some())
                    {
                        commands.trigger(HsdMeshGeometrySet {
                            doc_id,
                            id,
                            source: MeshGeometrySource::Hsd(Box::new(hsd_mesh)),
                        });
                    }
                }
                RawHsdChange::MeshChanged { id } => {
                    if let Some(hsd_mesh) = get_mesh_at(&hsd_map, &id) {
                        commands.trigger(HsdMeshGeometrySet {
                            doc_id,
                            id,
                            source: MeshGeometrySource::Hsd(Box::new(hsd_mesh)),
                        });
                    }
                }
                RawHsdChange::MeshRemoved { id } => {
                    commands.trigger(HsdMeshDespawned { doc_id, id });
                }
                RawHsdChange::MaterialAdded { id } => {
                    let initial = get_material_at(&hsd_map, &id);
                    commands.trigger(HsdMaterialSpawned {
                        doc_id,
                        id: id.clone(),
                        initial: initial.clone(),
                    });
                    if let Some(hsd_mat) = &initial {
                        emit_material_fields(doc_id, &id, hsd_mat, &mut commands);
                    }
                }
                RawHsdChange::MaterialChanged { id } => {
                    if let Some(hsd_mat) = get_material_at(&hsd_map, &id) {
                        emit_material_fields(doc_id, &id, &hsd_mat, &mut commands);
                    }
                }
                RawHsdChange::MaterialRemoved { id } => {
                    commands.trigger(HsdMaterialDespawned { doc_id, id });
                }
            }
        }
    }
}

fn emit_node_fields(doc_id: blake3::Hash, id: &SmolStr, data: &HsdNode, commands: &mut Commands) {
    commands.trigger(HsdNodeTransformSet {
        doc_id,
        id: id.clone(),
        transform: node_transform(data),
    });
    commands.trigger(HsdNodeMeshSet {
        doc_id,
        id: id.clone(),
        mesh: data.mesh.clone(),
    });
    commands.trigger(HsdNodeMaterialSet {
        doc_id,
        id: id.clone(),
        material: data.material.clone(),
    });
    if let Some(name) = &data.name {
        commands.trigger(HsdNodeNameSet {
            doc_id,
            id: id.clone(),
            name: Some(name.to_string()),
        });
    }
    commands.trigger(HsdNodeColliderSet {
        doc_id,
        id: id.clone(),
        collider: data.collider.clone(),
    });
    commands.trigger(HsdNodeRigidBodySet {
        doc_id,
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
        doc_id,
        id: id.clone(),
        scripts,
    });
}

fn emit_material_fields(
    doc_id: blake3::Hash,
    id: &SmolStr,
    hsd: &HsdMaterial,
    commands: &mut Commands,
) {
    if let Some(color) = &hsd.base_color
        && color.len() >= 3
    {
        let r = color[0] as f32;
        let g = color[1] as f32;
        let b = color[2] as f32;
        let a = color.get(3).copied().unwrap_or(1.0) as f32;
        commands.trigger(HsdMaterialBaseColorSet {
            doc_id,
            id: id.clone(),
            color: [r, g, b, a],
        });
    }
    if let Some(ref img_id) = hsd.base_color_texture {
        commands.trigger(HsdMaterialBaseColorTextureSet {
            doc_id,
            id: id.clone(),
            value: img_id.clone(),
        });
    }
    if let Some(ref img_id) = hsd.normal_texture {
        commands.trigger(HsdMaterialNormalTextureSet {
            doc_id,
            id: id.clone(),
            value: img_id.clone(),
        });
    }
    if let Some(ref img_id) = hsd.metallic_roughness_texture {
        commands.trigger(HsdMaterialMetallicRoughnessTextureSet {
            doc_id,
            id: id.clone(),
            value: img_id.clone(),
        });
    }
    if let Some(ref img_id) = hsd.occlusion_texture {
        commands.trigger(HsdMaterialOcclusionTextureSet {
            doc_id,
            id: id.clone(),
            value: img_id.clone(),
        });
    }
    if let Some(ref img_id) = hsd.emissive_texture {
        commands.trigger(HsdMaterialEmissiveTextureSet {
            doc_id,
            id: id.clone(),
            value: img_id.clone(),
        });
    }
    if let Some(v) = hsd.metallic {
        commands.trigger(HsdMaterialMetallicSet {
            doc_id,
            id: id.clone(),
            value: v as f32,
        });
    }
    if let Some(v) = hsd.roughness {
        commands.trigger(HsdMaterialRoughnessSet {
            doc_id,
            id: id.clone(),
            value: v as f32,
        });
    }
    if let Some(v) = hsd.alpha_cutoff {
        commands.trigger(HsdMaterialAlphaCutoffSet {
            doc_id,
            id: id.clone(),
            value: v as f32,
        });
    }
    if let Some(ref mode) = hsd.alpha_mode {
        commands.trigger(HsdMaterialAlphaModeSet {
            doc_id,
            id: id.clone(),
            mode: Some(mode.to_string()),
        });
    }
    if let Some(v) = hsd.double_sided {
        commands.trigger(HsdMaterialDoubleSidedSet {
            doc_id,
            id: id.clone(),
            value: v,
        });
    }
    if let Some(ref name) = hsd.name {
        commands.trigger(HsdMaterialNameSet {
            doc_id,
            id: id.clone(),
            name: Some(name.to_string()),
        });
    }
    if let Some(v) = hsd.unlit {
        commands.trigger(HsdMaterialUnlitSet {
            doc_id,
            id: id.clone(),
            value: v,
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

pub(super) fn node_data_from_hsd(hsd_map: &LoroMap, tid: TreeID) -> HsdNode {
    let tree = hsd_map
        .get_or_create_container("nodes", LoroTree::new())
        .ok();
    tree.as_ref()
        .and_then(|t| t.get_meta(tid).ok())
        .and_then(|m| HsdNode::hydrate(&m.get_deep_value()).ok())
        .unwrap_or_default()
}

pub(super) fn get_image_at(hsd_map: &LoroMap, key: &str) -> Option<HsdImage> {
    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return None;
    };
    let LoroValue::Map(map) = root.get("images")? else {
        return None;
    };
    HsdImage::hydrate(map.get(key)?).ok()
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
