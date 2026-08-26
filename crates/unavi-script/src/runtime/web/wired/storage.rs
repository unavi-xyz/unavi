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
    web::wired::{
        raise,
        variant_obj,
    },
};

#[wasm_bindgen]
pub struct StorageHandle {
    rep: u32,
    api: Arc<Api>,
}

impl StorageHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for StorageHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::storage::storage_drop(&api, rep).await;
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
                let _ = shared::wired::storage::get_future_drop(&api, rep).await;
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
                let _ = shared::wired::storage::list_future_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
impl StorageHandle {
    pub async fn get(&self, ns: Vec<u8>, key: String) -> GetFutureHandle {
        let rep = shared::wired::storage::get(&self.api, self.rep, ns, key)
            .await
            .unwrap_or(u32::MAX);
        GetFutureHandle::new(rep, Arc::clone(&self.api))
    }

    pub async fn list(&self, ns: Vec<u8>, prefix: String) -> ListFutureHandle {
        let rep = shared::wired::storage::list(&self.api, self.rep, ns, prefix)
            .await
            .unwrap_or(u32::MAX);
        ListFutureHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "rootDoc")]
    pub async fn root_doc(&self) -> Option<Vec<u8>> {
        shared::wired::storage::root_doc_ns(&self.api, self.rep)
            .ok()
            .flatten()
    }

    pub async fn registries(&self) -> JsValue {
        let Ok(namespaces) = shared::wired::storage::registry_namespaces(&self.api, self.rep)
        else {
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
        let Ok(Some(result)) = shared::wired::storage::get_future_poll(&self.api, self.rep).await
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
        let Ok(Some(result)) = shared::wired::storage::list_future_poll(&self.api, self.rep).await
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
impl Runtime {
    #[wasm_bindgen(js_name = "wiredStorageClass")]
    #[must_use]
    pub fn wired_storage_class(&self) -> JsValue {
        let handle = StorageHandle::new(u32::MAX, Arc::clone(&self.api));
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

    #[wasm_bindgen(js_name = "wiredStorageGetStorage")]
    pub async fn wired_storage_get_storage(&self) -> Result<StorageHandle, JsValue> {
        self.api.require(ApiName::Storage).map_err(raise)?;
        let rep = shared::wired::storage::get_storage(&self.api)
            .await
            .map_err(raise)?;
        Ok(StorageHandle::new(rep, Arc::clone(&self.api)))
    }
}
