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
    web::wired::raise,
};

#[wasm_bindgen]
pub struct KvHandle {
    rep: u32,
    api: Arc<Api>,
}

impl KvHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for KvHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::kv::kv_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
impl KvHandle {
    pub async fn get(&self, key: String) -> JsValue {
        match shared::wired::kv::kv_get(&self.api, self.rep, key).await {
            Ok(Some(bytes)) => js_sys::Uint8Array::from(bytes.as_slice()).into(),
            _ => JsValue::UNDEFINED,
        }
    }

    pub async fn set(&self, key: String, value: Vec<u8>) -> Result<(), JsValue> {
        shared::wired::kv::kv_set(&self.api, self.rep, key, value)
            .await
            .map_err(raise)?
            .map_err(raise)
    }

    pub async fn delete(&self, key: String) -> Result<(), JsValue> {
        shared::wired::kv::kv_delete(&self.api, self.rep, key)
            .await
            .map_err(raise)?
            .map_err(raise)
    }

    pub async fn keys(&self) -> JsValue {
        let keys = shared::wired::kv::kv_keys(&self.api, self.rep)
            .await
            .unwrap_or_default();
        keys.into_iter()
            .map(JsValue::from)
            .collect::<js_sys::Array>()
            .into()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredKvClass")]
    #[must_use]
    pub fn wired_kv_class(&self) -> JsValue {
        let handle = KvHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredKvSelfKv")]
    pub async fn wired_kv_self_kv(&self) -> Result<KvHandle, JsValue> {
        self.api.require(ApiName::Kv).map_err(raise)?;
        let rep = shared::wired::kv::self_kv(&self.api).await.map_err(raise)?;
        Ok(KvHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredKvGetKv")]
    pub async fn wired_kv_get_kv(&self, id: Vec<u8>) -> Result<Option<KvHandle>, JsValue> {
        self.api.require(ApiName::Kv).map_err(raise)?;
        let rep = shared::wired::kv::get_kv(&self.api, id)
            .await
            .map_err(raise)?;
        Ok(rep.map(|rep| KvHandle::new(rep, Arc::clone(&self.api))))
    }
}
