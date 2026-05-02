use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

pub mod document;
pub mod material;
pub mod mesh;
pub mod node;

use document::DocHandle;
use material::MaterialHandle;
use mesh::MeshHandle;
use node::NodeHandle;

#[wasm_bindgen]
impl Runtime {
    pub fn wired_scene_doc_class(&self) -> JsValue {
        let handle = DocHandle::new(u32::MAX, Arc::clone(&self.backend.wired_scene));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_scene_material_class(&self) -> JsValue {
        let handle = MaterialHandle::new(u32::MAX, Arc::clone(&self.backend.wired_scene));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_scene_mesh_class(&self) -> JsValue {
        let handle = MeshHandle::new(u32::MAX, Arc::clone(&self.backend.wired_scene));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_scene_node_class(&self) -> JsValue {
        let handle = NodeHandle::new(u32::MAX, Arc::clone(&self.backend.wired_scene));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_scene_self_node(&self) -> NodeHandle {
        let rep = self
            .backend
            .wired_scene
            .try_lock()
            .expect("no contention")
            .self_node();
        NodeHandle::new(rep, Arc::clone(&self.backend.wired_scene))
    }

    pub fn wired_scene_self_document(&self) -> DocHandle {
        let rep = self
            .backend
            .wired_scene
            .try_lock()
            .expect("no contention")
            .self_document();
        DocHandle::new(rep, Arc::clone(&self.backend.wired_scene))
    }

    pub async fn wired_scene_create_document(&self) -> Result<DocHandle, String> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .create_document()
            .await
            .map_err(|e| e.to_string())?;
        Ok(DocHandle::new(rep, Arc::clone(&self.backend.wired_scene)))
    }

    pub async fn wired_scene_get_document(&self, id: Vec<u8>) -> Option<DocHandle> {
        let rep = self.backend.wired_scene.lock().await.get_document(id)?;
        Some(DocHandle::new(rep, Arc::clone(&self.backend.wired_scene)))
    }

    pub async fn wired_scene_remove_document(&self, id: Vec<u8>) {
        self.backend.wired_scene.lock().await.remove_document(id);
    }

    pub async fn wired_scene_load_hsd(&self, blob_id: Vec<u8>) -> Result<DocHandle, String> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .load_hsd(blob_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DocHandle::new(rep, Arc::clone(&self.backend.wired_scene)))
    }
}
