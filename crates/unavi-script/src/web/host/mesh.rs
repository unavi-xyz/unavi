use std::sync::Arc;
use std::sync::atomic::Ordering;

use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};

use super::js_convert::{f32_from_js, js_f32_array, js_u32_array};
use super::state::MeshEntry;
use super::with_script;
use crate::core_ops;

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostSceneMeshDrop",
        dyn Fn(u32, u32),
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.meshes.remove(&rep);
            });
        }
    );

    reg!(
        obj,
        "hostSceneMeshId",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state
                    .meshes
                    .get(&rep)
                    .map(|entry| JsValue::from_str(&entry.inner.id))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneMeshClone",
        dyn Fn(u32, u32) -> u32,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let inner = Arc::clone(&state.meshes.get(&rep)?.inner);
                let doc_id = state.meshes.get(&rep)?.doc_id;
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
        "hostSceneMeshName",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.meshes.get(&rep)?;
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
        "hostSceneMeshSetName",
        dyn Fn(u32, u32, JsValue),
        |id: u32, rep: u32, value: JsValue| {
            with_script(id, |state| {
                let inner = Arc::clone(&state.meshes.get(&rep)?.inner);
                core_ops::mesh::set_name(&inner, value.as_string());
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneMeshTopology",
        dyn Fn(u32, u32) -> u32,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.meshes.get(&rep).and_then(|entry| {
                    entry.inner.state.lock().ok().map(|locked| {
                        use bevy::mesh::PrimitiveTopology::*;
                        match locked.topology {
                            PointList => 0,
                            LineList => 1,
                            LineStrip => 2,
                            TriangleList => 3,
                            TriangleStrip => 4,
                        }
                    })
                })
            })
            .flatten()
            .unwrap_or(3)
        }
    );

    reg!(
        obj,
        "hostSceneMeshSetTopology",
        dyn Fn(u32, u32, u32),
        |id: u32, rep: u32, value: u32| {
            with_script(id, |state| {
                use bevy::mesh::PrimitiveTopology::*;
                let topo = match value {
                    0 => PointList,
                    1 => LineList,
                    2 => LineStrip,
                    4 => TriangleStrip,
                    _ => TriangleList,
                };
                let inner = Arc::clone(&state.meshes.get(&rep)?.inner);
                core_ops::mesh::set_topology(&inner, topo);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneMeshIndices",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.meshes.get(&rep)?;
                let locked = entry.inner.state.lock().ok()?;
                let indices = locked.indices.as_ref()?;
                let arr = js_u32_array(indices);
                let obj = Object::new();
                js_sys::Reflect::set(&obj, &"tag".into(), &JsValue::from_str("full")).ok();
                js_sys::Reflect::set(&obj, &"val".into(), &arr).ok();
                Some(JsValue::from(obj))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneMeshSetIndices",
        dyn Fn(u32, u32, JsValue),
        |id: u32, rep: u32, value: JsValue| {
            with_script(id, |state| {
                let entry = state.meshes.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc_id = entry.doc_id;
                let indices = if value.is_null() || value.is_undefined() {
                    None
                } else {
                    let val = js_sys::Reflect::get(&value, &"val".into()).ok()?;
                    let tag = js_sys::Reflect::get(&value, &"tag".into()).ok()?;
                    let tag = tag.as_string().unwrap_or_default();
                    Some(if tag == "half" {
                        let arr = val.dyn_ref::<js_sys::Uint16Array>()?;
                        arr.to_vec().into_iter().map(u32::from).collect()
                    } else {
                        val.dyn_ref::<js_sys::Uint32Array>()?.to_vec()
                    })
                };
                core_ops::mesh::set_indices(&inner, doc_id, indices, &mut state.command_queue);
                Some(())
            });
        }
    );

    macro_rules! mesh_float_attr {
        ($obj:expr, $getter:literal, $setter:literal, $field:ident, $setter_fn:path) => {
            reg!(
                $obj,
                $getter,
                dyn Fn(u32, u32) -> JsValue,
                |id: u32, rep: u32| {
                    with_script(id, |state| {
                        let entry = state.meshes.get(&rep)?;
                        let locked = entry.inner.state.lock().ok()?;
                        locked.$field.as_deref().map(js_f32_array)
                    })
                    .flatten()
                    .unwrap_or(JsValue::NULL)
                }
            );
            reg!(
                $obj,
                $setter,
                dyn Fn(u32, u32, JsValue),
                |id: u32, rep: u32, value: JsValue| {
                    with_script(id, |state| {
                        let entry = state.meshes.get(&rep)?;
                        let inner = Arc::clone(&entry.inner);
                        let doc_id = entry.doc_id;
                        $setter_fn(
                            &inner,
                            doc_id,
                            f32_from_js(&value),
                            &mut state.command_queue,
                        );
                        Some(())
                    });
                }
            );
        };
    }

    mesh_float_attr!(
        obj,
        "hostSceneMeshPositions",
        "hostSceneMeshSetPositions",
        positions,
        core_ops::mesh::set_positions
    );
    mesh_float_attr!(
        obj,
        "hostSceneMeshNormals",
        "hostSceneMeshSetNormals",
        normals,
        core_ops::mesh::set_normals
    );
    mesh_float_attr!(
        obj,
        "hostSceneMeshTangents",
        "hostSceneMeshSetTangents",
        tangents,
        core_ops::mesh::set_tangents
    );
    mesh_float_attr!(
        obj,
        "hostSceneMeshColors",
        "hostSceneMeshSetColors",
        colors,
        core_ops::mesh::set_colors
    );
    mesh_float_attr!(
        obj,
        "hostSceneMeshUv0",
        "hostSceneMeshSetUv0",
        uv0,
        core_ops::mesh::set_uv0
    );
    mesh_float_attr!(
        obj,
        "hostSceneMeshUv1",
        "hostSceneMeshSetUv1",
        uv1,
        core_ops::mesh::set_uv1
    );

    reg!(
        obj,
        "hostSceneMeshSync",
        dyn Fn(u32, u32) -> bool,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state
                    .meshes
                    .get(&rep)
                    .map(|entry| entry.inner.sync.load(Ordering::Relaxed))
            })
            .flatten()
            .unwrap_or(false)
        }
    );

    reg!(
        obj,
        "hostSceneMeshSetSync",
        dyn Fn(u32, u32, bool),
        |id: u32, rep: u32, value: bool| {
            with_script(id, |state| {
                state
                    .meshes
                    .get(&rep)?
                    .inner
                    .sync
                    .store(value, Ordering::Relaxed);
                Some(())
            });
        }
    );
}
