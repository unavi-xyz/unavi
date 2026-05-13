use std::sync::Arc;

use wasm_bindgen::{JsValue, prelude::*};

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

fn scope_to_js(scope: SenderScope, api: &Arc<Api>) -> JsValue {
    let obj = js_sys::Object::new();
    match scope {
        SenderScope::Global => {
            js_sys::Reflect::set(&obj, &"tag".into(), &"global".into()).ok();
        }
        SenderScope::Spatial { distance, node } => {
            let node_rep = api.wired_scene.try_lock().ok().map_or(u32::MAX, |mut scene| {
                scene.nodes.insert(NodeRes {
                    doc: Arc::clone(&api.doc),
                    doc_id: node.doc,
                    id: node.node,
                    is_proxy: true,
                })
            });
            let val = js_sys::Object::new();
            js_sys::Reflect::set(&val, &"distance".into(), &distance.into()).ok();
            js_sys::Reflect::set(
                &val,
                &"node".into(),
                &JsValue::from(NodeHandle::new(node_rep, Arc::clone(api))),
            )
            .ok();
            js_sys::Reflect::set(&obj, &"tag".into(), &"spatial".into()).ok();
            js_sys::Reflect::set(&obj, &"val".into(), &val.into()).ok();
        }
    }
    obj.into()
}

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
    let get = |k: &str| {
        js_sys::Reflect::get(value, &k.into())
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
    };

    let scope = get("scope")
        .and_then(|v| {
            let tag = js_sys::Reflect::get(&v, &"tag".into())
                .ok()
                .and_then(|t| t.as_string())?;
            match tag.as_str() {
                "spatial" => {
                    let val = js_sys::Reflect::get(&v, &"val".into()).ok()?;
                    let radius = js_sys::Reflect::get(&val, &"radius".into())
                        .ok()
                        .and_then(|r| r.as_f64())
                        .unwrap_or(0.0) as f32;
                    let node = js_sys::Reflect::get(&val, &"node".into())
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
            return JsValue::UNDEFINED;
        };

        let sender_doc: js_sys::Uint8Array = event.sender_document.as_slice().into();
        let payload: js_sys::Uint8Array = event.payload.as_slice().into();

        let sender = js_sys::Object::new();
        js_sys::Reflect::set(&sender, &"document".into(), &sender_doc.into()).ok();
        js_sys::Reflect::set(
            &sender,
            &"scope".into(),
            &scope_to_js(event.sender_scope, &self.api),
        )
        .ok();

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"channel".into(), &event.channel.into()).ok();
        js_sys::Reflect::set(&obj, &"payload".into(), &payload.into()).ok();
        js_sys::Reflect::set(&obj, &"sender".into(), &sender.into()).ok();
        js_sys::Reflect::set(&obj, &"time".into(), &js_sys::BigInt::from(event.time).into()).ok();

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
