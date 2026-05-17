use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::{Runtime, shared};

pub mod document;
pub mod prim;
pub mod util;

use document::DocHandle;
use prim::PrimHandle;

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredSceneDocClass")]
    pub fn wired_scene_doc_class(&self) -> JsValue {
        let handle = DocHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredScenePrimClass")]
    pub fn wired_scene_prim_class(&self) -> JsValue {
        let handle = PrimHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredSceneSelfPrim")]
    pub fn wired_scene_self_prim(&self) -> PrimHandle {
        let rep = shared::wired::scene::self_prim(&self.api).unwrap_or(u32::MAX);
        PrimHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "wiredSceneSelfDocument")]
    pub fn wired_scene_self_document(&self) -> DocHandle {
        let rep = shared::wired::scene::self_document(&self.api).unwrap_or(u32::MAX);
        DocHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "wiredSceneGetDocument")]
    pub async fn wired_scene_get_document(&self, id: Vec<u8>) -> Option<DocHandle> {
        let rep = shared::wired::scene::get_document(&self.api, id)
            .await
            .ok()??;
        Some(DocHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredSceneCreateDocument")]
    pub async fn wired_scene_create_document(&self) -> Result<DocHandle, String> {
        let rep = shared::wired::scene::create_document(&self.api)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DocHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredSceneRemoveDocument")]
    pub fn wired_scene_remove_document(&self, id: Vec<u8>) -> Result<(), String> {
        shared::wired::scene::remove_document(&self.api, id).map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "wiredSceneLoadHsd")]
    pub async fn wired_scene_load_hsd(&self, blob_id: Vec<u8>) -> Result<DocHandle, String> {
        let rep = shared::wired::scene::load_hsd(&self.api, blob_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DocHandle::new(rep, Arc::clone(&self.api)))
    }
}
