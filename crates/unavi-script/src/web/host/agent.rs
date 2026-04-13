use std::sync::Arc;

use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};

use super::state::NodeEntry;
use super::with_script;

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostAgentContextLocalAgent",
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
                        doc_id: state.doc_id,
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
        "hostAgentContextLocalCamera",
        dyn Fn(u32) -> JsValue,
        |id: u32| {
            with_script(id, |state| {
                let inner = state
                    .registry
                    .node_map
                    .lock()
                    .ok()?
                    .get(&state.camera_node_id)
                    .cloned()?;
                let rep = state.alloc();
                state.nodes.insert(
                    rep,
                    NodeEntry {
                        inner,
                        doc_id: state.doc_id,
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
        "hostAgentBone",
        dyn Fn(u32, u32, u32) -> JsValue,
        |_id: u32, _agent_rep: u32, _bone_name: u32| {
            // Bone tracking not yet wired up in web runtime.
            JsValue::NULL
        }
    );
}
