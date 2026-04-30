use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;

use crate::runtime::shared::wired::scene::WiredSceneBackend;

use super::{material::MaterialHandle, mesh::MeshHandle, node::NodeHandle};

#[wasm_bindgen]
pub struct DocHandle {
    rep: u32,
    backend: Arc<Mutex<WiredSceneBackend>>,
}

impl DocHandle {
    pub const fn new(rep: u32, backend: Arc<Mutex<WiredSceneBackend>>) -> Self {
        Self { rep, backend }
    }
}

#[wasm_bindgen]
impl DocHandle {
    pub fn add_asset(&self, _name: String, _blob_id: Vec<u8>) {
        todo!()
    }

    pub fn assets(&self) -> JsValue {
        todo!()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_doc(&self) -> Self {
        todo!()
    }

    pub fn create_material(&self) -> MaterialHandle {
        todo!()
    }

    pub fn create_mesh(&self) -> MeshHandle {
        todo!()
    }

    pub fn create_node(&self) -> NodeHandle {
        todo!()
    }

    pub fn global_transform(&self) -> JsValue {
        todo!()
    }

    pub fn id(&self) -> Vec<u8> {
        self.backend
            .lock()
            .expect("lock")
            .docs
            .get(self.rep)
            .map(|d| d.id.as_bytes().to_vec())
            .unwrap_or_default()
    }

    pub fn materials(&self) -> JsValue {
        todo!()
    }

    pub fn meshes(&self) -> JsValue {
        todo!()
    }

    pub fn nodes(&self) -> JsValue {
        todo!()
    }

    pub fn public(&self) -> bool {
        todo!()
    }

    pub fn remove_asset(&self, _name: String) {
        todo!()
    }

    pub fn remove_material(&self, _value: JsValue) {
        todo!()
    }

    pub fn remove_mesh(&self, _value: JsValue) {
        todo!()
    }

    pub fn remove_node(&self, _value: JsValue) {
        todo!()
    }

    pub fn roots(&self) -> JsValue {
        todo!()
    }

    pub fn rotation(&self) -> JsValue {
        todo!()
    }

    pub fn scale(&self) -> JsValue {
        todo!()
    }

    pub fn set_public(&self, _value: bool) {
        todo!()
    }

    pub fn set_rotation(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_scale(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_sync(&self, _value: bool) {
        todo!()
    }

    pub fn set_transform(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_translation(&self, _value: JsValue) {
        todo!()
    }

    pub fn sync(&self) -> bool {
        todo!()
    }

    pub fn transform(&self) -> JsValue {
        todo!()
    }

    pub fn translation(&self) -> JsValue {
        todo!()
    }
}
