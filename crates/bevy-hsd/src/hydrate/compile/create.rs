use async_channel::Sender;
use bevy::prelude::*;
use loro::{LoroMap, LoroTree, TreeID, TreeParentId};
use smol_str::SmolStr;

use crate::{DocRegistryMap, HsdDoc};

#[derive(Event)]
pub struct HsdCreateNode {
    pub doc_id: blake3::Hash,
    pub parent_id: Option<TreeID>,
    pub tx: Sender<TreeID>,
}

#[derive(Event)]
pub struct HsdCreateMesh {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdCreateMaterial {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdRemoveNode {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
}

#[derive(Event)]
pub struct HsdRemoveMesh {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdRemoveMaterial {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
}

pub(crate) fn handle_hsd_create_node(
    trigger: On<HsdCreateNode>,
    registry_map: Res<DocRegistryMap>,
    docs: Query<&HsdDoc>,
) {
    let ev = trigger.event();
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(hsd_doc) = docs.get(doc_ent) else {
        return;
    };
    let doc = &hsd_doc.0;
    let hsd_map = doc.get_map("hsd");
    let Ok(tree) = hsd_map.get_or_create_container("nodes", LoroTree::new()) else {
        return;
    };
    let parent = ev.parent_id.map_or(TreeParentId::Root, TreeParentId::Node);
    let Ok(tree_id) = tree.create(parent) else {
        return;
    };
    doc.commit();
    ev.tx.try_send(tree_id).ok();
}

pub(crate) fn handle_hsd_create_mesh(
    trigger: On<HsdCreateMesh>,
    registry_map: Res<DocRegistryMap>,
    docs: Query<&HsdDoc>,
) {
    let ev = trigger.event();
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(hsd_doc) = docs.get(doc_ent) else {
        return;
    };
    let doc = &hsd_doc.0;
    let hsd_map = doc.get_map("hsd");
    let Ok(meshes_map) = hsd_map.get_or_create_container("meshes", LoroMap::new()) else {
        return;
    };
    let Ok(mesh_map) = meshes_map.get_or_create_container(ev.id.as_str(), LoroMap::new()) else {
        return;
    };
    // topology is required; 3 = TriangleList
    if mesh_map.insert("topology", 3i64).is_ok() {
        doc.commit();
    }
}

pub(crate) fn handle_hsd_create_material(
    trigger: On<HsdCreateMaterial>,
    registry_map: Res<DocRegistryMap>,
    docs: Query<&HsdDoc>,
) {
    let ev = trigger.event();
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(hsd_doc) = docs.get(doc_ent) else {
        return;
    };
    let doc = &hsd_doc.0;
    let hsd_map = doc.get_map("hsd");
    let Ok(materials_map) = hsd_map.get_or_create_container("materials", LoroMap::new()) else {
        return;
    };
    if materials_map
        .get_or_create_container(ev.id.as_str(), LoroMap::new())
        .is_ok()
    {
        doc.commit();
    }
}

pub(crate) fn handle_hsd_remove_node(
    trigger: On<HsdRemoveNode>,
    registry_map: Res<DocRegistryMap>,
    docs: Query<&HsdDoc>,
) {
    let ev = trigger.event();
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(hsd_doc) = docs.get(doc_ent) else {
        return;
    };
    let doc = &hsd_doc.0;
    let hsd_map = doc.get_map("hsd");
    let Ok(tree) = hsd_map.get_or_create_container("nodes", LoroTree::new()) else {
        return;
    };
    if tree.delete(ev.id).is_ok() {
        doc.commit();
    }
}

pub(crate) fn handle_hsd_remove_mesh(
    trigger: On<HsdRemoveMesh>,
    registry_map: Res<DocRegistryMap>,
    docs: Query<&HsdDoc>,
) {
    let ev = trigger.event();
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(hsd_doc) = docs.get(doc_ent) else {
        return;
    };
    let doc = &hsd_doc.0;
    let hsd_map = doc.get_map("hsd");
    let Ok(meshes_map) = hsd_map.get_or_create_container("meshes", LoroMap::new()) else {
        return;
    };
    if meshes_map
        .insert(ev.id.as_str(), loro::LoroValue::Null)
        .is_ok()
    {
        doc.commit();
    }
}

pub(crate) fn handle_hsd_remove_material(
    trigger: On<HsdRemoveMaterial>,
    registry_map: Res<DocRegistryMap>,
    docs: Query<&HsdDoc>,
) {
    let ev = trigger.event();
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(hsd_doc) = docs.get(doc_ent) else {
        return;
    };
    let doc = &hsd_doc.0;
    let hsd_map = doc.get_map("hsd");
    let Ok(materials_map) = hsd_map.get_or_create_container("materials", LoroMap::new()) else {
        return;
    };
    if materials_map
        .insert(ev.id.as_str(), loro::LoroValue::Null)
        .is_ok()
    {
        doc.commit();
    }
}
