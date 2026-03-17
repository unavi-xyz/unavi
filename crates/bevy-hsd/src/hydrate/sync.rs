use std::sync::Arc;

use bevy::prelude::*;
use loro::{LoroDoc, LoroList, LoroMap, LoroTree, LoroValue, TreeParentId};
use smol_str::ToSmolStr;

use crate::HsdDoc;
use crate::cache::{SceneRegistry, SyncOp};

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

pub(crate) fn sync_to_hsd(docs: Query<(&HsdDoc, &SceneRegistry)>) {
    for (hsd_doc, registry) in &docs {
        let doc = &hsd_doc.0;
        let wrote = process_sync_ops(doc, registry)
            | sync_node_changes(doc, registry)
            | sync_mesh_changes(doc, registry)
            | sync_material_changes(doc, registry);

        if wrote {
            doc.commit();
            doc.compact_change_store();
            doc.free_history_cache();
        }
    }
}

fn process_sync_ops(doc: &LoroDoc, registry: &SceneRegistry) -> bool {
    let mut wrote = false;

    let ops: Vec<SyncOp> = registry
        .0
        .pending_doc_ops
        .lock()
        .expect("pending doc ops lock")
        .drain(..)
        .collect();

    for op in ops {
        match op {
            SyncOp::NodeCreated(id) => {
                wrote |= handle_node_created(doc, registry, &id);
            }
            SyncOp::NodeRemoved(id) => {
                wrote |= handle_node_removed(doc, registry, &id);
            }
            SyncOp::MeshCreated(id) => {
                ensure_map_entry(doc, "meshes", id.as_str());
                wrote = true;
            }
            SyncOp::MeshRemoved(id) => {
                delete_map_entry(doc, "meshes", id.as_str());
                wrote = true;
            }
            SyncOp::MaterialCreated(id) => {
                ensure_map_entry(doc, "materials", id.as_str());
                wrote = true;
            }
            SyncOp::MaterialRemoved(id) => {
                delete_map_entry(doc, "materials", id.as_str());
                wrote = true;
            }
        }
    }

    wrote
}

fn handle_node_created(doc: &LoroDoc, registry: &SceneRegistry, id: &str) -> bool {
    let inner = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(id)
        .cloned();
    let Some(inner) = inner else { return false };
    let mut lock = inner.tree_id.lock().expect("lock tree id");
    if lock.is_some() {
        return false;
    }
    let hsd_map = doc.get_map("hsd");
    let Ok(tree) = hsd_map.get_or_create_container("nodes", LoroTree::new()) else {
        return false;
    };
    let Ok(tid) = tree.create(TreeParentId::Root) else {
        return false;
    };
    *lock = Some(tid);
    drop(lock);
    registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .insert(tid.to_smolstr(), Arc::clone(&inner));
    true
}

fn handle_node_removed(doc: &LoroDoc, registry: &SceneRegistry, id: &str) -> bool {
    let inner = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(id)
        .cloned();
    let Some(inner) = inner else { return false };
    let Some(tid) = *inner.tree_id.lock().expect("lock tree id") else {
        return false;
    };
    let hsd_map = doc.get_map("hsd");
    let Ok(tree) = hsd_map.get_or_create_container("nodes", LoroTree::new()) else {
        return false;
    };
    let _ = tree.delete(tid);
    true
}

fn ensure_map_entry(doc: &LoroDoc, section: &str, id: &str) {
    let hsd_map = doc.get_map("hsd");
    if let Ok(map) = hsd_map.get_or_create_container(section, LoroMap::new()) {
        let _ = map.get_or_create_container(id, LoroMap::new());
    }
}

fn delete_map_entry(doc: &LoroDoc, section: &str, id: &str) {
    let hsd_map = doc.get_map("hsd");
    if let Ok(map) = hsd_map.get_or_create_container(section, LoroMap::new()) {
        let _ = map.delete(id);
    }
}

fn sync_node_changes(doc: &LoroDoc, registry: &SceneRegistry) -> bool {
    let mut wrote = false;
    {
        let nodes = registry.0.nodes.lock().expect("nodes lock");
        for inner in nodes.iter() {
            if inner.is_virtual {
                continue;
            }
            let ch = inner.hsd_changes.lock().expect("hsd_changes lock");
            if ch.is_empty() {
                continue;
            }

            let hsd_map = doc.get_map("hsd");
            let Ok(tree) = hsd_map.get_or_create_container("nodes", LoroTree::new()) else {
                continue;
            };

            let tree_id = {
                let mut lock = inner.tree_id.lock().expect("lock tree id");
                if let Some(tid) = *lock {
                    tid
                } else {
                    let Ok(tid) = tree.create(TreeParentId::Root) else {
                        continue;
                    };
                    *lock = Some(tid);
                    drop(lock);
                    registry
                        .0
                        .node_map
                        .lock()
                        .expect("node_map lock")
                        .insert(tid.to_smolstr(), Arc::clone(inner));
                    tid
                }
            };

            let Ok(meta) = tree.get_meta(tree_id) else {
                continue;
            };

            if let Some(t) = ch.translation {
                write_f64s(&meta, "translation", &t);
                wrote = true;
            }
            if let Some(r) = ch.rotation {
                write_f64s(&meta, "rotation", &r);
                wrote = true;
            }
            if let Some(s) = ch.scale {
                write_f64s(&meta, "scale", &s);
                wrote = true;
            }
            if let Some(ref name_opt) = ch.name {
                if let Some(n) = name_opt {
                    let _ = meta.insert("name", n.as_str());
                }
                wrote = true;
            }
            if let Some(ref mesh_opt) = ch.mesh {
                if let Some(id) = mesh_opt {
                    let _ = meta.insert("mesh", id.to_string());
                }
                wrote = true;
            }
            if let Some(ref mat_opt) = ch.material {
                if let Some(id) = mat_opt {
                    let _ = meta.insert("material", id.to_string());
                }
                wrote = true;
            }
        }
    }
    wrote
}

fn sync_mesh_changes(doc: &LoroDoc, registry: &SceneRegistry) -> bool {
    let mut wrote = false;
    {
        let meshes = registry.0.meshes.lock().expect("meshes lock");
        for inner in meshes.values() {
            let ch = inner.hsd_changes.lock().expect("hsd_changes lock");
            if ch.is_empty() {
                continue;
            }

            let hsd_map = doc.get_map("hsd");
            let Ok(meshes_map) = hsd_map.get_or_create_container("meshes", LoroMap::new()) else {
                continue;
            };
            let Ok(map) = meshes_map.get_or_create_container(inner.id.as_str(), LoroMap::new())
            else {
                continue;
            };

            if let Some(topo) = ch.topology {
                let _ = map.insert("topology", topo);
                wrote = true;
            }
            if let Some(ref name_opt) = ch.name {
                if let Some(n) = name_opt {
                    let _ = map.insert("name", n.as_str());
                }
                wrote = true;
            }
        }
    }
    wrote
}

fn sync_material_changes(doc: &LoroDoc, registry: &SceneRegistry) -> bool {
    let mut wrote = false;
    {
        let materials = registry.0.materials.lock().expect("materials lock");
        for inner in materials.values() {
            let ch = inner.hsd_changes.lock().expect("hsd_changes lock");
            if ch.is_empty() {
                continue;
            }

            let hsd_map = doc.get_map("hsd");
            let Ok(mats_map) = hsd_map.get_or_create_container("materials", LoroMap::new()) else {
                continue;
            };
            let Ok(map) = mats_map.get_or_create_container(inner.id.as_str(), LoroMap::new())
            else {
                continue;
            };

            if let Some(cutoff) = ch.alpha_cutoff {
                let _ = map.insert("alpha_cutoff", cutoff);
                wrote = true;
            }
            if let Some(ref mode_opt) = ch.alpha_mode {
                if let Some(mode) = mode_opt {
                    let _ = map.insert("alpha_mode", mode.as_str());
                }
                wrote = true;
            }
            if let Some(base_color) = ch.base_color {
                if let Ok(list) = map.get_or_create_container("base_color", LoroList::new()) {
                    let len = list.len();
                    if len > 0 {
                        let _ = list.delete(0, len);
                    }
                    for v in base_color {
                        let _ = list.push(LoroValue::Double(v));
                    }
                }
                wrote = true;
            }
            if let Some(ds) = ch.double_sided {
                let _ = map.insert("double_sided", ds);
                wrote = true;
            }
            if let Some(m) = ch.metallic {
                let _ = map.insert("metallic", m);
                wrote = true;
            }
            if let Some(ref name_opt) = ch.name {
                if let Some(n) = name_opt {
                    let _ = map.insert("name", n.as_str());
                }
                wrote = true;
            }
            if let Some(r) = ch.roughness {
                let _ = map.insert("roughness", r);
                wrote = true;
            }
        }
    }
    wrote
}

fn write_f64s(meta: &LoroMap, key: &str, vals: &[f64]) {
    let Ok(list) = meta.get_or_create_container(key, LoroList::new()) else {
        return;
    };
    let len = list.len();
    if len > 0 {
        let _ = list.delete(0, len);
    }
    for &v in vals {
        let _ = list.push(LoroValue::Double(v));
    }
}
