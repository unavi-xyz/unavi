use std::sync::Arc;

use unavi_space::state::doc::KvError;
use unavi_util::async_task::spawn_async_task;
use wasm_bindgen::prelude::*;

use crate::{
    permissions::ApiName,
    runtime::{
        Runtime,
        shared::{
            self,
            Api,
        },
    },
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

fn variant_obj(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"tag".into(), &tag.into()).ok();
    js_sys::Reflect::set(&obj, &"val".into(), &val).ok();
    obj.into()
}

fn kv_error_tag(e: KvError) -> &'static str {
    match e {
        KvError::QuotaExceeded => "quota-exceeded",
        KvError::KeyTooLong => "key-too-long",
        KvError::Other => "other",
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

    pub async fn set(&self, key: String, value: Vec<u8>) -> JsValue {
        match shared::wired::kv::kv_set(&self.api, self.rep, key, value).await {
            Ok(Ok(())) => variant_obj("ok", JsValue::UNDEFINED),
            Ok(Err(e)) => variant_obj("err", variant_obj(kv_error_tag(e), JsValue::UNDEFINED)),
            Err(_) => variant_obj("err", variant_obj("other", JsValue::UNDEFINED)),
        }
    }

    pub async fn delete(&self, key: String) {
        let _ = shared::wired::kv::kv_delete(&self.api, self.rep, key).await;
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
    pub fn wired_kv_class(&self) -> JsValue {
        let handle = KvHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredKvSelfKv")]
    pub async fn wired_kv_self_kv(&self) -> KvHandle {
        let rep = match self.api.require(ApiName::Kv) {
            Ok(()) => shared::wired::kv::self_kv(&self.api)
                .await
                .unwrap_or(u32::MAX),
            Err(_) => u32::MAX,
        };
        KvHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "wiredKvGetKv")]
    pub async fn wired_kv_get_kv(&self, id: Vec<u8>) -> JsValue {
        if self.api.require(ApiName::Kv).is_err() {
            return JsValue::UNDEFINED;
        }
        match shared::wired::kv::get_kv(&self.api, id).await {
            Ok(Some(rep)) => JsValue::from(KvHandle::new(rep, Arc::clone(&self.api))),
            _ => JsValue::UNDEFINED,
        }
    }
}
