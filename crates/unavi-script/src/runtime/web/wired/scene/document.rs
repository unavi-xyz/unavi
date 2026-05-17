use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::shared::{self, Api};

use super::prim::PrimHandle;

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
            let _ = shared::wired::scene::document::on_drop(&self.api, self.rep);
        }
    }
}

#[wasm_bindgen]
impl DocHandle {
    pub fn id(&self) -> Vec<u8> {
        shared::wired::scene::document::id(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_doc(&self) -> Option<Self> {
        let rep = shared::wired::scene::document::clone(&self.api, self.rep).ok()?;
        Some(Self::new(rep, Arc::clone(&self.api)))
    }

    pub fn roots(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::roots(&self.api, self.rep) else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(PrimHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    pub fn prims(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::prims(&self.api, self.rep) else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(PrimHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    #[wasm_bindgen(js_name = "getPrim")]
    pub fn get_prim(&self, id: String) -> Option<PrimHandle> {
        let rep = shared::wired::scene::document::get_prim(&self.api, self.rep, id).ok()??;
        Some(PrimHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "createPrim")]
    pub fn create_prim(&self) -> PrimHandle {
        let rep = shared::wired::scene::document::create_prim(&self.api, self.rep)
            .unwrap_or(u32::MAX);
        PrimHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "removePrim")]
    pub fn remove_prim(&self, value: &PrimHandle) -> Result<(), String> {
        shared::wired::scene::document::remove_prim(&self.api, value.rep())
            .map_err(|e| e.to_string())
    }
}
