use std::sync::Arc;

use js_sys::Object;
use wasm_bindgen::JsValue;

use super::state::{DocEntry, NodeEntry};
use super::with_script;

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostSceneContextSelfNode",
        dyn Fn(u32) -> JsValue,
        |id: u32| {
            with_script(id, |state| {
                let inner = state
                    .registry
                    .node_map
                    .lock()
                    .ok()?
                    .get(&state.self_node_id)
                    .cloned()?;
                let rep = state.alloc();
                state.nodes.insert(
                    rep,
                    NodeEntry {
                        inner,
                        doc_entity: state.doc_entity,
                    },
                );
                Some(JsValue::from_f64(f64::from(rep)))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneContextSelfDocument",
        dyn Fn(u32) -> u32,
        |id: u32| {
            with_script(id, |state| {
                let rep = state.alloc();
                state.docs.insert(
                    rep,
                    DocEntry {
                        id: state.doc_id,
                        registry: Arc::clone(&state.registry),
                        doc_entity: state.doc_entity,
                    },
                );
                rep
            })
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostSceneContextGetDocument",
        dyn Fn(u32, JsValue) -> JsValue,
        |id: u32, id_bytes: JsValue| {
            let bytes = id_bytes.dyn_ref::<js_sys::Uint8Array>().map(|a| a.to_vec());
            let hash = bytes
                .as_deref()
                .and_then(|b| b.try_into().ok())
                .map(blake3::Hash::from);
            let hash = match hash {
                Some(h) => h,
                None => return JsValue::NULL,
            };
            with_script(id, |state| {
                if hash == state.doc_id {
                    let rep = state.alloc();
                    state.docs.insert(
                        rep,
                        DocEntry {
                            id: state.doc_id,
                            registry: Arc::clone(&state.registry),
                            doc_entity: state.doc_entity,
                        },
                    );
                    Some(JsValue::from_f64(f64::from(rep)))
                } else {
                    None
                }
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );
}
