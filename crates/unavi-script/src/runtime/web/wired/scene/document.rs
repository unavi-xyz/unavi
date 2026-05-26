use std::sync::Arc;

use unavi_util::async_task::spawn_async_task;
use wasm_bindgen::prelude::*;

use super::prim::PrimHandle;
use crate::runtime::shared::{
    self,
    Api,
};

#[wasm_bindgen]
pub struct DocHandle {
    rep: u32,
    api: Arc<Api>,
}

impl DocHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for DocHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::scene::document::on_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
impl DocHandle {
    pub async fn id(&self) -> Vec<u8> {
        shared::wired::scene::document::id(&self.api, self.rep)
            .await
            .unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub async fn clone_doc(&self) -> Option<Self> {
        let rep = shared::wired::scene::document::clone(&self.api, self.rep)
            .await
            .ok()?;
        Some(Self::new(rep, Arc::clone(&self.api)))
    }

    pub async fn roots(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::roots(&self.api, self.rep).await else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(PrimHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    pub async fn prims(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::prims(&self.api, self.rep).await else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(PrimHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    #[wasm_bindgen(js_name = "getPrim")]
    pub async fn get_prim(&self, id: String) -> Option<PrimHandle> {
        let rep = shared::wired::scene::document::get_prim(&self.api, self.rep, id)
            .await
            .ok()??;
        Some(PrimHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "createPrim")]
    pub async fn create_prim(&self) -> PrimHandle {
        let rep = shared::wired::scene::document::create_prim(&self.api, self.rep)
            .await
            .unwrap_or(u32::MAX);
        PrimHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "removePrim")]
    pub async fn remove_prim(&self, value: &PrimHandle) -> Result<(), String> {
        shared::wired::scene::document::remove_prim(&self.api, value.rep())
            .await
            .map_err(|e| e.to_string())
    }
}
