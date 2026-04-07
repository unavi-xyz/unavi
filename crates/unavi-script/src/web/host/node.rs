use std::sync::Arc;
use std::sync::atomic::Ordering;

use bevy::math::Vec3 as BVec3;
use bevy::prelude::Transform;
use bevy_hsd::cache::NodeInner;
use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};

use super::js_convert::{js_quat, js_transform, js_u32_array, js_vec3};
use super::state::{MatEntry, MeshEntry, NodeEntry};
use super::with_script;
use crate::core_ops;

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostSceneNodeDrop",
        dyn Fn(u32, u32),
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.nodes.remove(&rep);
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeId",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state
                    .nodes
                    .get(&rep)
                    .map(|entry| JsValue::from_str(&entry.inner.id))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeClone",
        dyn Fn(u32, u32) -> u32,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let inner = Arc::clone(&state.nodes.get(&rep)?.inner);
                let doc_entity = state.nodes.get(&rep)?.doc_entity;
                let new_rep = state.alloc();
                state.nodes.insert(
                    new_rep,
                    NodeEntry {
                        inner,
                        doc_entity,
                    },
                );
                Some(new_rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostSceneNodeName",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let locked = entry.inner.state.lock().ok()?;
                Some(
                    locked
                        .name
                        .as_deref()
                        .map(JsValue::from_str)
                        .unwrap_or(JsValue::NULL),
                )
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeSetName",
        dyn Fn(u32, u32, JsValue),
        |id: u32, rep: u32, value: JsValue| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                core_ops::node::set_name(&inner, doc, value.as_string(), &mut state.command_queue);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeTranslation",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let translation = state
                    .nodes
                    .get(&rep)?
                    .inner
                    .state
                    .lock()
                    .ok()?
                    .transform
                    .translation;
                Some(js_vec3(translation.x, translation.y, translation.z))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeSetTranslation",
        dyn Fn(u32, u32, f64, f64, f64),
        |id: u32, rep: u32, x: f64, y: f64, z: f64| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                core_ops::node::set_translation(
                    &inner,
                    doc,
                    x as f32,
                    y as f32,
                    z as f32,
                    &mut state.command_queue,
                );
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeRotation",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let rotation = state
                    .nodes
                    .get(&rep)?
                    .inner
                    .state
                    .lock()
                    .ok()?
                    .transform
                    .rotation;
                Some(js_quat(rotation.x, rotation.y, rotation.z, rotation.w))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeSetRotation",
        dyn Fn(u32, u32, f64, f64, f64, f64),
        |id: u32, rep: u32, x: f64, y: f64, z: f64, w: f64| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                core_ops::node::set_rotation(
                    &inner,
                    doc,
                    x as f32,
                    y as f32,
                    z as f32,
                    w as f32,
                    &mut state.command_queue,
                );
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeScale",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let scale = state
                    .nodes
                    .get(&rep)?
                    .inner
                    .state
                    .lock()
                    .ok()?
                    .transform
                    .scale;
                Some(js_vec3(scale.x, scale.y, scale.z))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeSetScale",
        dyn Fn(u32, u32, f64, f64, f64),
        |id: u32, rep: u32, x: f64, y: f64, z: f64| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                core_ops::node::set_scale(
                    &inner,
                    doc,
                    x as f32,
                    y as f32,
                    z as f32,
                    &mut state.command_queue,
                );
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeTransform",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let transform = state.nodes.get(&rep)?.inner.state.lock().ok()?.transform;
                Some(js_transform(&transform))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeSetTransform",
        dyn Fn(u32, u32, JsValue),
        |id: u32, rep: u32, vals: JsValue| {
            let floats = super::js_convert::parse_f32_array(&vals);
            if floats.len() < 10 {
                return;
            }
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                let new_transform = Transform {
                    translation: BVec3::new(floats[0], floats[1], floats[2]),
                    rotation: bevy::math::Quat::from_xyzw(
                        floats[3], floats[4], floats[5], floats[6],
                    ),
                    scale: BVec3::new(floats[7], floats[8], floats[9]),
                };
                core_ops::node::set_transform(&inner, doc, new_transform, &mut state.command_queue);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeGlobalTransform",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let global = state
                    .nodes
                    .get(&rep)?
                    .inner
                    .state
                    .lock()
                    .ok()?
                    .global_transform;
                let (scale, rotation, translation) = global.to_scale_rotation_translation();
                let transform = Transform {
                    translation,
                    rotation,
                    scale,
                };
                Some(js_transform(&transform))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeParent",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let parent = {
                    let locked = entry.inner.state.lock().ok()?;
                    locked.parent.as_ref()?.upgrade()?
                };
                let doc = entry.doc_entity;
                let new_rep = state.alloc();
                state.nodes.insert(
                    new_rep,
                    NodeEntry {
                        inner: parent,
                        doc_entity: doc,
                    },
                );
                Some(JsValue::from_f64(f64::from(new_rep)))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeChildren",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let doc = entry.doc_entity;
                let children: Vec<Arc<NodeInner>> =
                    { entry.inner.state.lock().ok()?.children.clone() };
                let mut reps = Vec::with_capacity(children.len());
                for child in children {
                    let new_rep = state.alloc();
                    state.nodes.insert(
                        new_rep,
                        NodeEntry {
                            inner: child,
                            doc_entity: doc,
                        },
                    );
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
        "hostSceneNodeAddChild",
        dyn Fn(u32, u32, u32),
        |id: u32, rep: u32, child_rep: u32| {
            with_script(id, |state| {
                let (parent_inner, doc) = {
                    let entry = state.nodes.get(&rep)?;
                    (Arc::clone(&entry.inner), entry.doc_entity)
                };
                let child_inner = state
                    .nodes
                    .get(&child_rep)
                    .map(|entry| Arc::clone(&entry.inner))?;
                core_ops::node::add_child(
                    &parent_inner,
                    &child_inner,
                    doc,
                    &mut state.command_queue,
                );
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeRemoveChild",
        dyn Fn(u32, u32, u32),
        |id: u32, _rep: u32, child_rep: u32| {
            with_script(id, |state| {
                let (child_inner, doc) = {
                    let entry = state.nodes.get(&child_rep)?;
                    (Arc::clone(&entry.inner), entry.doc_entity)
                };
                core_ops::node::remove_child(&child_inner, doc, &mut state.command_queue);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeMesh",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let mesh_id = entry.inner.state.lock().ok()?.mesh.clone()?;
                let mesh_inner = state.registry.meshes.lock().ok()?.get(&mesh_id).cloned()?;
                let doc = entry.doc_entity;
                let new_rep = state.alloc();
                state.meshes.insert(
                    new_rep,
                    MeshEntry {
                        inner: mesh_inner,
                        doc_entity: doc,
                    },
                );
                Some(JsValue::from_f64(f64::from(new_rep)))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeSetMesh",
        dyn Fn(u32, u32, u32),
        |id: u32, rep: u32, mesh_rep: u32| {
            with_script(id, |state| {
                let (node_inner, doc) = {
                    let entry = state.nodes.get(&rep)?;
                    (Arc::clone(&entry.inner), entry.doc_entity)
                };
                let mesh_id = if mesh_rep == 0 {
                    None
                } else {
                    state
                        .meshes
                        .get(&mesh_rep)
                        .map(|entry| entry.inner.id.clone())
                };
                core_ops::node::set_mesh(&node_inner, doc, mesh_id, &mut state.command_queue);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeMaterial",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.nodes.get(&rep)?;
                let mat_id = entry.inner.state.lock().ok()?.material.clone()?;
                let mat_inner = state
                    .registry
                    .materials
                    .lock()
                    .ok()?
                    .get(&mat_id)
                    .cloned()?;
                let doc = entry.doc_entity;
                let new_rep = state.alloc();
                state.mats.insert(
                    new_rep,
                    MatEntry {
                        inner: mat_inner,
                        doc_entity: doc,
                    },
                );
                Some(JsValue::from_f64(f64::from(new_rep)))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneNodeSetMaterial",
        dyn Fn(u32, u32, u32),
        |id: u32, rep: u32, mat_rep: u32| {
            with_script(id, |state| {
                let (node_inner, doc) = {
                    let entry = state.nodes.get(&rep)?;
                    (Arc::clone(&entry.inner), entry.doc_entity)
                };
                let mat_id = if mat_rep == 0 {
                    None
                } else {
                    state.mats.get(&mat_rep).map(|entry| entry.inner.id.clone())
                };
                core_ops::node::set_material(&node_inner, doc, mat_id, &mut state.command_queue);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneNodeSync",
        dyn Fn(u32, u32) -> bool,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state
                    .nodes
                    .get(&rep)
                    .map(|entry| entry.inner.sync.load(Ordering::Relaxed))
            })
            .flatten()
            .unwrap_or(false)
        }
    );

    reg!(
        obj,
        "hostSceneNodeSetSync",
        dyn Fn(u32, u32, bool),
        |id: u32, rep: u32, value: bool| {
            with_script(id, |state| {
                state
                    .nodes
                    .get(&rep)?
                    .inner
                    .sync
                    .store(value, Ordering::Relaxed);
                Some(())
            });
        }
    );
}
