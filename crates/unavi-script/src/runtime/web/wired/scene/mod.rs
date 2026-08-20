use std::sync::Arc;

use unavi_policy::document::ApiName;
use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared,
    web::wired::raise,
};

pub mod document;
pub mod prim;
pub mod shader_graph;
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
    pub async fn wired_scene_self_prim(&self) -> Result<PrimHandle, JsValue> {
        self.api.require(ApiName::Scene).map_err(raise)?;
        let rep = shared::wired::scene::self_prim(&self.api)
            .await
            .map_err(raise)?;
        Ok(PrimHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredSceneSelfDocument")]
    pub async fn wired_scene_self_document(&self) -> Result<DocHandle, JsValue> {
        self.api.require(ApiName::Scene).map_err(raise)?;
        let rep = shared::wired::scene::self_document(&self.api)
            .await
            .map_err(raise)?;
        Ok(DocHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredSceneGetDocument")]
    pub async fn wired_scene_get_document(
        &self,
        id: Vec<u8>,
    ) -> Result<Option<DocHandle>, JsValue> {
        self.api.require(ApiName::Scene).map_err(raise)?;
        let rep = shared::wired::scene::get_document(&self.api, id)
            .await
            .map_err(raise)?;
        Ok(rep.map(|rep| DocHandle::new(rep, Arc::clone(&self.api))))
    }

    #[wasm_bindgen(js_name = "wiredSceneCreateDocument")]
    pub async fn wired_scene_create_document(&self) -> Result<DocHandle, JsValue> {
        self.api.require(ApiName::CreateDocument).map_err(raise)?;
        let rep = shared::wired::scene::create_document(&self.api)
            .await
            .map_err(raise)?;
        Ok(DocHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredSceneRemoveDocument")]
    pub async fn wired_scene_remove_document(&self, id: Vec<u8>) -> Result<(), JsValue> {
        self.api.require(ApiName::Scene).map_err(raise)?;
        shared::wired::scene::remove_document(&self.api, id)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredSceneCreateDocumentFromPrefab")]
    pub async fn wired_scene_create_document_from_prefab(
        &self,
        prefab: Vec<u8>,
    ) -> Result<DocHandle, JsValue> {
        self.api.require(ApiName::CreateDocument).map_err(raise)?;
        let rep = shared::wired::scene::create_document_from_prefab(&self.api, prefab)
            .await
            .map_err(raise)?;
        Ok(DocHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "wiredSceneSyncDocument")]
    pub async fn wired_scene_sync_document(&self, id: Vec<u8>) -> Result<(), JsValue> {
        self.api.require(ApiName::CreateDocument).map_err(raise)?;
        shared::wired::scene::sync_document(&self.api, id)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredSceneSaveDocument")]
    pub async fn wired_scene_save_document(&self, id: Vec<u8>) -> Result<(), JsValue> {
        self.api.require(ApiName::Scene).map_err(raise)?;
        shared::wired::scene::save_document(&self.api, id)
            .await
            .map_err(raise)
    }
}
