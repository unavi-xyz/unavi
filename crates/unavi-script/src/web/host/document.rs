use std::sync::Arc;
use std::sync::atomic::Ordering;

use bevy_hsd::cache::NodeInner;
use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};

use super::js_convert::js_u32_array;
use super::state::{DocEntry, MatEntry, MeshEntry, NodeEntry};
use super::with_script;
use crate::core_ops;

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostSceneDocumentDrop",
        dyn Fn(u32, u32),
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.docs.remove(&rep);
            });
        }
    );

    reg!(
        obj,
        "hostSceneDocumentId",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.docs.get(&rep).map(|entry| {
                    let arr = js_sys::Uint8Array::from(entry.id.as_bytes().as_slice());
                    JsValue::from(arr)
                })
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneDocumentClone",
        dyn Fn(u32, u32) -> u32,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.docs.get(&rep)?;
                let clone = DocEntry {
                    id: entry.id,
                    registry: Arc::clone(&entry.registry),
                    is_public: entry.is_public,
                    can_read: entry.can_read,
                    can_write: entry.can_write,
                };
                let new_rep = state.alloc();
                state.docs.insert(new_rep, clone);
                Some(new_rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostSceneDocumentCreateNode",
        dyn Fn(u32, u32) -> u32,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let (registry, doc_id) = {
                    let entry = state.docs.get(&rep)?;
                    (Arc::clone(&entry.registry), entry.id)
                };
                let inner =
                    core_ops::document::create_node(&registry, doc_id, &mut state.command_queue);
                let new_rep = state.alloc();
                state.nodes.insert(new_rep, NodeEntry { inner, doc_id });
                Some(new_rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostSceneDocumentCreateMesh",
        dyn Fn(u32, u32) -> u32,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let (registry, doc_id) = {
                    let entry = state.docs.get(&rep)?;
                    (Arc::clone(&entry.registry), entry.id)
                };
                let inner =
                    core_ops::document::create_mesh(&registry, doc_id, &mut state.command_queue);
                let new_rep = state.alloc();
                state.meshes.insert(new_rep, MeshEntry { inner, doc_id });
                Some(new_rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostSceneDocumentCreateMaterial",
        dyn Fn(u32, u32) -> u32,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let (registry, doc_id) = {
                    let entry = state.docs.get(&rep)?;
                    (Arc::clone(&entry.registry), entry.id)
                };
                let inner = core_ops::document::create_material(
                    &registry,
                    doc_id,
                    &mut state.command_queue,
                );
                let new_rep = state.alloc();
                state.mats.insert(new_rep, MatEntry { inner, doc_id });
                Some(new_rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostSceneDocumentRoots",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let (registry, doc) = {
                    let entry = state.docs.get(&rep)?;
                    (Arc::clone(&entry.registry), entry.id)
                };
                let nodes: Vec<Arc<NodeInner>> = registry
                    .nodes
                    .lock()
                    .ok()?
                    .iter()
                    .filter(|node| {
                        node.state
                            .lock()
                            .ok()
                            .map(|locked| {
                                locked
                                    .parent
                                    .as_ref()
                                    .is_none_or(|weak| weak.upgrade().is_none())
                            })
                            .unwrap_or(true)
                    })
                    .cloned()
                    .collect();
                let mut reps = Vec::with_capacity(nodes.len());
                for inner in nodes {
                    let new_rep = state.alloc();
                    state
                        .nodes
                        .insert(new_rep, NodeEntry { inner, doc_id: doc });
                    reps.push(new_rep);
                }
                Some(js_u32_array(&reps))
            })
            .flatten()
            .unwrap_or_else(|| js_u32_array(&[]))
        }
    );

    reg!(
        obj,
        "hostSceneDocumentNodes",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let (registry, doc) = {
                    let entry = state.docs.get(&rep)?;
                    (Arc::clone(&entry.registry), entry.id)
                };
                let nodes: Vec<Arc<NodeInner>> = registry.nodes.lock().ok()?.clone();
                let mut reps = Vec::with_capacity(nodes.len());
                for inner in nodes {
                    let new_rep = state.alloc();
                    state
                        .nodes
                        .insert(new_rep, NodeEntry { inner, doc_id: doc });
                    reps.push(new_rep);
                }
                Some(js_u32_array(&reps))
            })
            .flatten()
            .unwrap_or_else(|| js_u32_array(&[]))
        }
    );

    reg!(
        obj,
        "hostSceneDocumentMeshes",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let (registry, doc) = {
                    let entry = state.docs.get(&rep)?;
                    (Arc::clone(&entry.registry), entry.id)
                };
                let meshes: Vec<_> = registry.meshes.lock().ok()?.values().cloned().collect();
                let mut reps = Vec::with_capacity(meshes.len());
                for inner in meshes {
                    let new_rep = state.alloc();
                    state
                        .meshes
                        .insert(new_rep, MeshEntry { inner, doc_id: doc });
                    reps.push(new_rep);
                }
                Some(js_u32_array(&reps))
            })
            .flatten()
            .unwrap_or_else(|| js_u32_array(&[]))
        }
    );

    reg!(
        obj,
        "hostSceneDocumentMaterials",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let (registry, doc) = {
                    let entry = state.docs.get(&rep)?;
                    (Arc::clone(&entry.registry), entry.id)
                };
                let mats: Vec<_> = registry.materials.lock().ok()?.values().cloned().collect();
                let mut reps = Vec::with_capacity(mats.len());
                for inner in mats {
                    let new_rep = state.alloc();
                    state.mats.insert(new_rep, MatEntry { inner, doc_id: doc });
                    reps.push(new_rep);
                }
                Some(js_u32_array(&reps))
            })
            .flatten()
            .unwrap_or_else(|| js_u32_array(&[]))
        }
    );

    reg!(
        obj,
        "hostSceneDocumentRemoveNode",
        dyn Fn(u32, u32, u32),
        |id: u32, rep: u32, node_rep: u32| {
            with_script(id, |state| {
                let (inner, doc, registry) = {
                    let node_entry = state.nodes.get(&node_rep)?;
                    let doc_entry = state.docs.get(&rep)?;
                    (
                        Arc::clone(&node_entry.inner),
                        node_entry.doc.clone(),
                        Arc::clone(&doc_entry.registry),
                    )
                };
                core_ops::document::remove_node(&inner, &registry, doc, &mut state.command_queue);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneDocumentRemoveMesh",
        dyn Fn(u32, u32, u32),
        |id: u32, rep: u32, mesh_rep: u32| {
            with_script(id, |state| {
                let (inner, doc, registry) = {
                    let mesh_entry = state.meshes.get(&mesh_rep)?;
                    let doc_entry = state.docs.get(&rep)?;
                    (
                        Arc::clone(&mesh_entry.inner),
                        mesh_entry.doc.clone(),
                        Arc::clone(&doc_entry.registry),
                    )
                };
                core_ops::document::remove_mesh(&inner, &registry, doc, &mut state.command_queue);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneDocumentRemoveMaterial",
        dyn Fn(u32, u32, u32),
        |id: u32, rep: u32, mat_rep: u32| {
            with_script(id, |state| {
                let (inner, doc, registry) = {
                    let mat_entry = state.mats.get(&mat_rep)?;
                    let doc_entry = state.docs.get(&rep)?;
                    (
                        Arc::clone(&mat_entry.inner),
                        mat_entry.doc.clone(),
                        Arc::clone(&doc_entry.registry),
                    )
                };
                core_ops::document::remove_material(
                    &inner,
                    &registry,
                    doc,
                    &mut state.command_queue,
                );
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneDocumentSync",
        dyn Fn(u32, u32) -> bool,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state
                    .docs
                    .get(&rep)
                    .map(|entry| entry.registry.doc_sync.load(Ordering::Relaxed))
            })
            .flatten()
            .unwrap_or(false)
        }
    );

    reg!(
        obj,
        "hostSceneDocumentSetSync",
        dyn Fn(u32, u32, bool),
        |id: u32, rep: u32, value: bool| {
            with_script(id, |state| {
                let registry = Arc::clone(&state.docs.get(&rep)?.registry);
                core_ops::document::set_sync(&registry, value);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneDocumentPublic",
        dyn Fn(u32, u32) -> bool,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.docs.get(&rep).map(|entry| entry.is_public)
            })
            .flatten()
            .unwrap_or(false)
        }
    );

    reg!(
        obj,
        "hostSceneDocumentSetPublic",
        dyn Fn(u32, u32, bool),
        |id: u32, rep: u32, value: bool| {
            with_script(id, |state| {
                if let Some(entry) = state.docs.get_mut(&rep) {
                    entry.is_public = value;
                }
            });
        }
    );
}
