use std::sync::Arc;

use unavi_util::async_task::spawn_async_task;
use wasm_bindgen::{
    JsValue,
    prelude::*,
};

use super::scene::{
    prim::PrimHandle,
    util::opt_rep,
};
use crate::{
    permissions::ApiName,
    runtime::{
        Runtime,
        shared::{
            self,
            Api,
            registry::event::SenderScope,
            wired::{
                event::{
                    EventFilter,
                    EventScope,
                },
                scene::prim::PrimRes,
            },
        },
    },
};

async fn scope_to_js(scope: SenderScope, api: &Arc<Api>) -> JsValue {
    let obj = js_sys::Object::new();
    match scope {
        SenderScope::Global => {
            js_sys::Reflect::set(&obj, &"tag".into(), &"global".into()).ok();
        }
        SenderScope::Spatial { distance, node } => {
            let inserted = api.wired_scene.lock().await.prims.insert(
                PrimRes {
                    doc:      Arc::clone(&api.doc),
                    doc_id:   node.doc,
                    id:       node.node,
                    is_proxy: true,
                },
                &api.quota,
            );
            let Ok(node_rep) = inserted else {
                js_sys::Reflect::set(&obj, &"tag".into(), &"global".into()).ok();
                return obj.into();
            };
            let val = js_sys::Object::new();
            js_sys::Reflect::set(&val, &"distance".into(), &distance.into()).ok();
            js_sys::Reflect::set(
                &val,
                &"node".into(),
                &JsValue::from(PrimHandle::new(node_rep, Arc::clone(api))),
            )
            .ok();
            js_sys::Reflect::set(&obj, &"tag".into(), &"spatial".into()).ok();
            js_sys::Reflect::set(&obj, &"val".into(), &val.into()).ok();
        }
    }
    obj.into()
}

#[wasm_bindgen]
pub struct EventHandle {
    rep: u32,
    api: Arc<Api>,
}

impl EventHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for EventHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::event::event_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
impl EventHandle {
    pub async fn channel(&self) -> String {
        shared::wired::event::event_clone_inner(&self.api, self.rep)
            .await
            .map(|e| e.channel)
            .unwrap_or_default()
    }

    pub async fn payload(&self) -> JsValue {
        match shared::wired::event::event_clone_inner(&self.api, self.rep).await {
            Ok(inner) => js_sys::Uint8Array::from(inner.payload.as_slice()).into(),
            Err(_) => JsValue::UNDEFINED,
        }
    }

    pub async fn sender(&self) -> JsValue {
        let Ok(inner) = shared::wired::event::event_clone_inner(&self.api, self.rep).await else {
            return JsValue::UNDEFINED;
        };
        let sender_doc: js_sys::Uint8Array = inner.sender_document.as_slice().into();
        let sender = js_sys::Object::new();
        js_sys::Reflect::set(&sender, &"document".into(), &sender_doc.into()).ok();
        js_sys::Reflect::set(
            &sender,
            &"scope".into(),
            &scope_to_js(inner.sender_scope, &self.api).await,
        )
        .ok();
        sender.into()
    }

    pub async fn time(&self) -> JsValue {
        match shared::wired::event::event_clone_inner(&self.api, self.rep).await {
            Ok(inner) => js_sys::BigInt::from(inner.time).into(),
            Err(_) => JsValue::UNDEFINED,
        }
    }

    pub async fn consume(&self) -> bool {
        shared::wired::event::event_consume(&self.api, self.rep)
            .await
            .unwrap_or(false)
    }
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
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::event::receptor_drop(&api, rep).await;
            });
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
    pub async fn poll(&self) -> JsValue {
        let Ok(Some(event)) = shared::wired::event::receptor_poll(&self.api, self.rep).await else {
            return JsValue::UNDEFINED;
        };
        let Ok(rep) = shared::wired::event::insert_event(&self.api, event).await else {
            return JsValue::UNDEFINED;
        };
        JsValue::from(EventHandle::new(rep, Arc::clone(&self.api)))
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

    #[wasm_bindgen(js_name = "wiredEventClass")]
    pub fn wired_event_class(&self) -> JsValue {
        let handle = EventHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredEventEmit")]
    pub async fn wired_event_emit(
        &self,
        channel: String,
        payload: Vec<u8>,
        filter: JsValue,
    ) -> Result<(), String> {
        self.api
            .require(ApiName::Event)
            .map_err(|e| e.to_string())?;
        shared::wired::event::emit(&self.api, channel, payload, js_to_event_filter(&filter))
            .await
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "wiredEventListen")]
    pub async fn wired_event_listen(
        &self,
        channels: JsValue,
        filter: JsValue,
    ) -> EventReceptorHandle {
        let rep = if self.api.require(ApiName::Event).is_ok() {
            let channels = js_sys::Array::from(&channels)
                .iter()
                .filter_map(|v| v.as_string())
                .collect();
            shared::wired::event::listen(&self.api, channels, js_to_event_filter(&filter))
                .await
                .unwrap_or(u32::MAX)
        } else {
            u32::MAX
        };
        EventReceptorHandle::new(rep, Arc::clone(&self.api))
    }
}
