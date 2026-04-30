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

    pub fn wired_scene_self_document(&self) -> DocHandle {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .expect("lock")
            .self_document();
        DocHandle::new(rep, Arc::clone(&self.backend.wired_scene))
    }

    pub fn wired_scene_self_node(&self) -> NodeHandle {
        let rep = self.backend.wired_scene.lock().expect("lock").self_node();
        NodeHandle::new(rep, Arc::clone(&self.backend.wired_scene))
    }

    pub fn wired_scene_create_document(&self) -> DocHandle {
        todo!()
    }

    pub fn wired_scene_get_document(&self, _id: Vec<u8>) -> Option<DocHandle> {
        todo!()
    }

    pub fn wired_scene_remove_document(&self, _id: Vec<u8>) {
        todo!()
    }

    pub fn wired_scene_load_hsd(&self, _blob_id: Vec<u8>) -> DocHandle {
        todo!()
    }
}
