use std::sync::Arc;

use unavi_policy::document::ApiName;
use unavi_util::async_task::spawn_async_task;
use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        Api,
    },
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
pub struct GetFutureHandle {
    rep: u32,
    api: Arc<Api>,
}

impl GetFutureHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for GetFutureHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::wds::get_future_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
pub struct ListFutureHandle {
    rep: u32,
    api: Arc<Api>,
}

impl ListFutureHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for ListFutureHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::wds::list_future_drop(&api, rep).await;
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

fn variant_obj(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"tag".into(), &tag.into()).ok();
    js_sys::Reflect::set(&obj, &"val".into(), &val).ok();
    obj.into()
}

#[wasm_bindgen]
impl WdsHandle {
    #[wasm_bindgen(js_name = "createDoc")]
    pub async fn create_doc(&self) -> Option<Vec<u8>> {
        shared::wired::wds::create_doc(&self.api, self.rep)
            .await
            .ok()
    }

    pub async fn set(&self, ns: Vec<u8>, key: String, value: Vec<u8>) {
        let _ = shared::wired::wds::set(&self.api, self.rep, ns, key, value).await;
    }

    pub async fn delete(&self, ns: Vec<u8>, key: String) {
        let _ = shared::wired::wds::delete(&self.api, self.rep, ns, key).await;
    }

    pub async fn get(&self, ns: Vec<u8>, key: String) -> GetFutureHandle {
        let rep = shared::wired::wds::get(&self.api, self.rep, ns, key)
            .await
            .unwrap_or(u32::MAX);
        GetFutureHandle::new(rep, Arc::clone(&self.api))
    }

    pub async fn list(&self, ns: Vec<u8>, prefix: String) -> ListFutureHandle {
        let rep = shared::wired::wds::list(&self.api, self.rep, ns, prefix)
            .await
            .unwrap_or(u32::MAX);
        ListFutureHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "getBlob")]
    pub async fn get_blob(&self, blob_id: Vec<u8>) -> BlobFutureHandle {
        let rep = shared::wired::wds::get_blob(&self.api, self.rep, blob_id)
            .await
            .unwrap_or(u32::MAX);
        BlobFutureHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "rootDoc")]
    pub async fn root_doc(&self) -> Option<Vec<u8>> {
        shared::wired::wds::root_doc_ns(&self.api, self.rep)
            .ok()
            .flatten()
    }

    pub async fn registries(&self) -> JsValue {
        let Ok(namespaces) = shared::wired::wds::registry_namespaces(&self.api, self.rep) else {
            return js_sys::Array::new().into();
        };
        namespaces
            .into_iter()
            .map(|ns| JsValue::from(js_sys::Uint8Array::from(ns.as_slice())))
            .collect::<js_sys::Array>()
            .into()
    }
}

#[wasm_bindgen]
impl GetFutureHandle {
    pub async fn poll(&self) -> JsValue {
        let Ok(Some(result)) = shared::wired::wds::get_future_poll(&self.api, self.rep).await
        else {
            return JsValue::UNDEFINED;
        };
        match result {
            Ok(Some(bytes)) => variant_obj("ok", js_sys::Uint8Array::from(bytes.as_slice()).into()),
            Ok(None) => variant_obj("ok", JsValue::UNDEFINED),
            Err(()) => variant_obj("err", JsValue::UNDEFINED),
        }
    }
}

#[wasm_bindgen]
impl ListFutureHandle {
    pub async fn poll(&self) -> JsValue {
        let Ok(Some(result)) = shared::wired::wds::list_future_poll(&self.api, self.rep).await
        else {
            return JsValue::UNDEFINED;
        };
        match result {
            Ok(entries) => {
                let arr: js_sys::Array = entries
                    .into_iter()
                    .map(|entry| {
                        let obj = js_sys::Object::new();
                        js_sys::Reflect::set(&obj, &"key".into(), &entry.key.into()).ok();
                        js_sys::Reflect::set(
                            &obj,
                            &"value".into(),
                            &js_sys::Uint8Array::from(entry.value.as_slice()).into(),
                        )
                        .ok();
                        JsValue::from(obj)
                    })
                    .collect();
                variant_obj("ok", arr.into())
            }
            Err(()) => variant_obj("err", JsValue::UNDEFINED),
        }
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
impl Runtime {
    #[wasm_bindgen(js_name = "wiredWdsClass")]
    #[must_use]
    pub fn wired_wds_class(&self) -> JsValue {
        let handle = WdsHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredGetFutureClass")]
    #[must_use]
    pub fn wired_get_future_class(&self) -> JsValue {
        let handle = GetFutureHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredListFutureClass")]
    #[must_use]
    pub fn wired_list_future_class(&self) -> JsValue {
        let handle = ListFutureHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredBlobFutureClass")]
    #[must_use]
    pub fn wired_blob_future_class(&self) -> JsValue {
        let handle = BlobFutureHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredWdsGetWds")]
    pub async fn wired_wds_get_wds(&self) -> WdsHandle {
        let rep = match self.api.require(ApiName::Wds) {
            Ok(()) => shared::wired::wds::get_wds(&self.api)
                .await
                .unwrap_or(u32::MAX),
            Err(_) => u32::MAX,
        };
        WdsHandle::new(rep, Arc::clone(&self.api))
    }
}
