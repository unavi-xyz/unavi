use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared::{
        self, Api,
        registry::event::SenderScope,
        wired::{
            event::{EventFilter, EventScope},
            scene::node::NodeRes,
        },
    },
};

use super::scene::{node::NodeHandle, util::opt_rep};

#[wasm_bindgen]
pub struct EventReceptorHandle {
    rep: u32,
    api: Arc<Api>,
}

impl EventReceptorHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for EventReceptorHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let _ = shared::wired::event::receptor_drop(&self.api, self.rep);
        }
    }
}

fn js_to_event_filter(value: &JsValue) -> EventFilter {
    let get = |k: &str| js_sys::Reflect::get(value, &k.into()).ok();

    let scope = get("scope")
        .and_then(|v| {
            let kind = js_sys::Reflect::get(&v, &"type".into())
                .ok()
                .and_then(|t| t.as_string())?;
            match kind.as_str() {
                "spatial" => {
                    let radius = js_sys::Reflect::get(&v, &"radius".into())
                        .ok()
                        .and_then(|r| r.as_f64())
                        .unwrap_or(0.0) as f32;
                    let node = js_sys::Reflect::get(&v, &"node".into())
                        .ok()
                        .and_then(|n| opt_rep(&n))
                        .unwrap_or(u32::MAX);
                    Some(EventScope::Spatial { node, radius })
                }
                _ => Some(EventScope::Global),
            }
        })
        .unwrap_or_default();

    let documents = get("documents").and_then(|v| {
        let arr = js_sys::Array::from(&v);
        let docs: Vec<Vec<u8>> = arr
            .iter()
            .map(|item| js_sys::Uint8Array::new(&item).to_vec())
            .collect();
        if docs.is_empty() { None } else { Some(docs) }
    });

    EventFilter { documents, scope }
}

#[wasm_bindgen]
impl EventReceptorHandle {
    pub fn poll(&self) -> JsValue {
        let Ok(Some(event)) = shared::wired::event::receptor_poll(&self.api, self.rep) else {
            return JsValue::NULL;
        };

        let obj = js_sys::Object::new();
        let payload: js_sys::Uint8Array = event.payload.as_slice().into();
        let sender_doc: js_sys::Uint8Array = event.sender_document.as_slice().into();

        js_sys::Reflect::set(&obj, &"channel".into(), &event.channel.into()).ok();
        js_sys::Reflect::set(&obj, &"payload".into(), &payload.into()).ok();
        js_sys::Reflect::set(&obj, &"senderDocument".into(), &sender_doc.into()).ok();
        js_sys::Reflect::set(&obj, &"time".into(), &(event.time as f64).into()).ok();

        match event.sender_scope {
            SenderScope::Global => {
                js_sys::Reflect::set(&obj, &"sender".into(), &"global".into()).ok();
            }
            SenderScope::Spatial { distance, node } => {
                let node_id = self.api.wired_scene.try_lock().ok().map(|mut scene| {
                    scene.nodes.insert(NodeRes {
                        doc: Arc::clone(&self.api.doc),
                        doc_id: node.doc,
                        id: node.node,
                        is_proxy: true,
                    })
                });
                js_sys::Reflect::set(&obj, &"sender".into(), &"spatial".into()).ok();
                js_sys::Reflect::set(&obj, &"senderDistance".into(), &distance.into()).ok();
                if let Some(id) = node_id {
                    let handle = NodeHandle::new(id, Arc::clone(&self.api));
                    js_sys::Reflect::set(&obj, &"senderNode".into(), &JsValue::from(handle)).ok();
                }
            }
        }

        obj.into()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredEventReceptorClass")]
    pub fn wired_event_receptor_class(&self) -> JsValue {
        let handle = EventReceptorHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredEventEmit")]
    pub fn wired_event_emit(
        &self,
        channel: String,
        payload: Vec<u8>,
        filter: JsValue,
    ) -> Result<(), String> {
        shared::wired::event::emit(&self.api, channel, payload, js_to_event_filter(&filter))
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "wiredEventListen")]
    pub fn wired_event_listen(&self, channels: JsValue, filter: JsValue) -> EventReceptorHandle {
        let channels = js_sys::Array::from(&channels)
            .iter()
            .filter_map(|v| v.as_string())
            .collect();
        let rep = shared::wired::event::listen(&self.api, channels, js_to_event_filter(&filter))
            .unwrap_or(u32::MAX);
        EventReceptorHandle::new(rep, Arc::clone(&self.api))
    }
}
