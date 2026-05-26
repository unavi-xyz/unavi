use std::sync::Arc;

use unavi_util::async_task::spawn_async_task;
use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared::{self, Api, wired::wds::QueryFilter},
};

#[wasm_bindgen]
pub struct WdsHandle {
    rep: u32,
    api: Arc<Api>,
}

impl WdsHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for WdsHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::wds::wds_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
pub struct QueryFutureHandle {
    rep: u32,
    api: Arc<Api>,
}

impl QueryFutureHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for QueryFutureHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::wds::query_future_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
pub struct ReadFutureHandle {
    rep: u32,
    api: Arc<Api>,
}

impl ReadFutureHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for ReadFutureHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::wds::read_future_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
pub struct BlobFutureHandle {
    rep: u32,
    api: Arc<Api>,
}

impl BlobFutureHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for BlobFutureHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::wds::blob_future_drop(&api, rep).await;
            });
        }
    }
}

fn js_to_query_filter(value: &JsValue) -> Option<QueryFilter> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    let get = |k: &str| {
        js_sys::Reflect::get(value, &k.into())
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
    };

    let creator = get("creator").and_then(|v| v.as_string());
    let schemas = get("schemas").map(|v| {
        js_sys::Array::from(&v)
            .iter()
            .map(|item| js_sys::Uint8Array::new(&item).to_vec())
            .collect()
    });

    Some(QueryFilter { creator, schemas })
}

fn variant_obj(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"tag".into(), &tag.into()).ok();
    js_sys::Reflect::set(&obj, &"val".into(), &val).ok();
    obj.into()
}

#[wasm_bindgen]
impl WdsHandle {
    pub async fn query(&self, filter: JsValue) -> QueryFutureHandle {
        let rep = shared::wired::wds::query(&self.api, self.rep, js_to_query_filter(&filter))
            .await
            .unwrap_or(u32::MAX);
        QueryFutureHandle::new(rep, Arc::clone(&self.api))
    }

    pub async fn read(&self, record_id: Vec<u8>) -> ReadFutureHandle {
        let rep = shared::wired::wds::read(&self.api, self.rep, record_id)
            .await
            .unwrap_or(u32::MAX);
        ReadFutureHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "getBlob")]
    pub async fn get_blob(&self, blob_id: Vec<u8>) -> BlobFutureHandle {
        let rep = shared::wired::wds::get_blob(&self.api, self.rep, blob_id)
            .await
            .unwrap_or(u32::MAX);
        BlobFutureHandle::new(rep, Arc::clone(&self.api))
    }
}

#[wasm_bindgen]
impl BlobFutureHandle {
    pub async fn poll(&self) -> JsValue {
        let Ok(Some(result)) = shared::wired::wds::blob_future_poll(&self.api, self.rep).await
        else {
            return JsValue::UNDEFINED;
        };
        match result {
            Ok(bytes) => variant_obj("ok", js_sys::Uint8Array::from(bytes.as_slice()).into()),
            Err(()) => variant_obj("err", JsValue::UNDEFINED),
        }
    }
}

#[wasm_bindgen]
impl QueryFutureHandle {
    pub async fn poll(&self) -> JsValue {
        let Ok(Some(result)) = shared::wired::wds::query_future_poll(&self.api, self.rep).await
        else {
            return JsValue::UNDEFINED;
        };
        match result {
            Ok(hashes) => {
                let arr: js_sys::Array = hashes
                    .iter()
                    .map(|h| {
                        let bytes: js_sys::Uint8Array = h.as_slice().into();
                        JsValue::from(bytes)
                    })
                    .collect();
                variant_obj("ok", arr.into())
            }
            Err(()) => variant_obj("err", JsValue::UNDEFINED),
        }
    }
}

#[wasm_bindgen]
impl ReadFutureHandle {
    pub async fn poll(&self) -> JsValue {
        let Ok(Some(result)) = shared::wired::wds::read_future_poll(&self.api, self.rep).await
        else {
            return JsValue::UNDEFINED;
        };
        match result {
            Ok(record) => {
                let record_obj = js_sys::Object::new();
                let id: js_sys::Uint8Array = record.id.as_slice().into();
                let schemas: js_sys::Array = record
                    .schemas
                    .iter()
                    .map(|h| JsValue::from(js_sys::Uint8Array::from(h.as_slice())))
                    .collect();
                let containers: js_sys::Array = record
                    .containers
                    .iter()
                    .map(|(k, v)| {
                        let pair = js_sys::Array::new();
                        pair.push(&k.into());
                        pair.push(&js_sys::Uint8Array::from(v.as_slice()).into());
                        JsValue::from(pair)
                    })
                    .collect();
                js_sys::Reflect::set(&record_obj, &"id".into(), &id.into()).ok();
                js_sys::Reflect::set(&record_obj, &"creator".into(), &record.creator.into()).ok();
                js_sys::Reflect::set(&record_obj, &"schemas".into(), &schemas.into()).ok();
                js_sys::Reflect::set(&record_obj, &"containers".into(), &containers.into()).ok();
                variant_obj("ok", record_obj.into())
            }
            Err(()) => variant_obj("err", JsValue::UNDEFINED),
        }
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredWdsClass")]
    pub fn wired_wds_class(&self) -> JsValue {
        let handle = WdsHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredQueryFutureClass")]
    pub fn wired_query_future_class(&self) -> JsValue {
        let handle = QueryFutureHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredReadFutureClass")]
    pub fn wired_read_future_class(&self) -> JsValue {
        let handle = ReadFutureHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredBlobFutureClass")]
    pub fn wired_blob_future_class(&self) -> JsValue {
        let handle = BlobFutureHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredWdsGetWds")]
    pub async fn wired_wds_get_wds(&self) -> WdsHandle {
        let rep = shared::wired::wds::get_wds(&self.api)
            .await
            .unwrap_or(u32::MAX);
        WdsHandle::new(rep, Arc::clone(&self.api))
    }
}
