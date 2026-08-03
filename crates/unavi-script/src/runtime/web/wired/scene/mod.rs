use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::{
    permissions::ApiName,
    runtime::{
        Runtime,
        shared,
    },
};

pub mod document;
pub mod prim;
pub mod util;

use document::DocHandle;
use prim::PrimHandle;

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredSceneDocClass")]
    #[must_use]
    pub fn wired_scene_doc_class(&self) -> JsValue {
        let handle = DocHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredScenePrimClass")]
    #[must_use]
    pub fn wired_scene_prim_class(&self) -> JsValue {
        let handle = PrimHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredSceneSelfPrim")]
    pub async fn wired_scene_self_prim(&self) -> PrimHandle {
        let rep = match self.api.require(ApiName::Scene) {
            Ok(()) => shared::wired::scene::self_prim(&self.api)
                .await
                .unwrap_or(u32::MAX),
            Err(_) => u32::MAX,
        };
        PrimHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "wiredSceneSelfDocument")]
    pub async fn wired_scene_self_document(&self) -> DocHandle {
        let rep = match self.api.require(ApiName::Scene) {
            Ok(()) => shared::wired::scene::self_document(&self.api)
                .await
                .unwrap_or(u32::MAX),
            Err(_) => u32::MAX,
        };
        DocHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "wiredSceneGetDocument")]
    pub async fn wired_scene_get_document(&self, id: Vec<u8>) -> Option<DocHandle> {
        self.api.require(ApiName::Scene).ok()?;
        let rep = shared::wired::scene::get_document(&self.api, id)
            .await
            .ok()??;
        Some(DocHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredSceneCreateDocument")]
    pub async fn wired_scene_create_document(&self) -> Result<DocHandle, String> {
        self.api
            .require(ApiName::CreateDocument)
            .map_err(|e| e.to_string())?;
        let rep = shared::wired::scene::create_document(&self.api)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DocHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredSceneRemoveDocument")]
    pub async fn wired_scene_remove_document(&self, id: Vec<u8>) -> Result<(), String> {
        self.api
            .require(ApiName::Scene)
            .map_err(|e| e.to_string())?;
        shared::wired::scene::remove_document(&self.api, id)
            .await
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "wiredSceneLoadHsd")]
    pub async fn wired_scene_load_hsd(&self, blob_id: Vec<u8>) -> Result<DocHandle, String> {
        self.api
            .require(ApiName::CreateDocument)
            .map_err(|e| e.to_string())?;
        let rep = shared::wired::scene::load_hsd(&self.api, blob_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DocHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredScenePublishDocument")]
    pub async fn wired_scene_sync_document(&self, id: Vec<u8>) -> Result<(), String> {
        self.api
            .require(ApiName::CreateDocument)
            .map_err(|e| e.to_string())?;
        shared::wired::scene::sync_document(&self.api, id)
            .await
            .map_err(|e| e.to_string())
    }
}
