use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared::{
        self, Api,
        wired::event::{EventFilter, EventScope},
    },
};

use super::scene::util::opt_rep;

#[wasm_bindgen]
pub struct EventReceptorHandle {
    rep: u32,
    api: Arc<Api>,
}

impl EventReceptorHandle {
    pub fn new(rep: u32, api: Arc<Api>) -> Self {
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

    let node = opt_rep(value);

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
                    Some(EventScope::Spatial(radius))
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

    EventFilter {
        node,
        scope,
        documents,
    }
}

#[wasm_bindgen]
impl EventReceptorHandle {
    pub fn poll(&self) -> JsValue {
        let Ok(Some(event)) = shared::wired::event::receptor_poll(&self.api, self.rep) else {
            return JsValue::NULL;
        };
        let obj = js_sys::Object::new();
        let sender = match event.sender {
            shared::wired::event::EventSender::Global => "global",
            shared::wired::event::EventSender::Spatial => "spatial",
        };
        let payload: js_sys::Uint8Array = event.payload.as_slice().into();
        let sender_doc: js_sys::Uint8Array = event.sender_document.as_slice().into();
        js_sys::Reflect::set(&obj, &"channel".into(), &event.channel.into()).ok();
        js_sys::Reflect::set(&obj, &"payload".into(), &payload.into()).ok();
        js_sys::Reflect::set(&obj, &"sender".into(), &sender.into()).ok();
        js_sys::Reflect::set(&obj, &"senderDocument".into(), &sender_doc.into()).ok();
        js_sys::Reflect::set(&obj, &"time".into(), &(event.time as f64).into()).ok();
        obj.into()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredEventReceptorClass")]
    pub fn wired_event_receptor_class(&self) -> JsValue {
        let handle = EventReceptorHandle::new(u32::MAX, self.api.clone());
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
        EventReceptorHandle::new(rep, self.api.clone())
    }
}
