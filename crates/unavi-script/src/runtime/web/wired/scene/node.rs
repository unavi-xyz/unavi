use std::sync::Arc;

use tokio::sync::Mutex;
use wasm_bindgen::prelude::*;

use crate::runtime::shared::wired::scene::WiredSceneBackend;

use super::{material::MaterialHandle, mesh::MeshHandle};

#[wasm_bindgen]
pub struct NodeHandle {
    rep: u32,
    backend: Arc<Mutex<WiredSceneBackend>>,
}

impl NodeHandle {
    pub const fn new(rep: u32, backend: Arc<Mutex<WiredSceneBackend>>) -> Self {
        Self { rep, backend }
    }

    pub fn rep(&self) -> u32 {
        self.rep
    }
}

#[wasm_bindgen]
impl NodeHandle {
    pub fn add_child(&self, _child: Self) {
        todo!()
    }

    pub fn children(&self) -> JsValue {
        todo!()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_node(&self) -> Self {
        todo!()
    }

    pub fn collider(&self) -> JsValue {
        todo!()
    }

    pub fn global_transform(&self) -> JsValue {
        todo!()
    }

    pub async fn id(&self) -> String {
        self.backend
            .lock()
            .await
            .nodes
            .get(self.rep)
            .map(|n| n.id.to_string())
            .unwrap_or_default()
    }

    pub fn material(&self) -> Option<MaterialHandle> {
        todo!()
    }

    pub fn mesh(&self) -> Option<MeshHandle> {
        todo!()
    }

    pub fn name(&self) -> Option<String> {
        todo!()
    }

    pub fn parent(&self) -> Option<Self> {
        todo!()
    }

    pub fn remove_child(&self, _child: Self) {
        todo!()
    }

    pub fn rigid_body(&self) -> JsValue {
        todo!()
    }

    pub fn rotation(&self) -> JsValue {
        todo!()
    }

    pub fn scale(&self) -> JsValue {
        todo!()
    }

    pub fn set_collider(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_material(&self, _value: Option<MaterialHandle>) {
        todo!()
    }

    pub fn set_mesh(&self, _value: Option<MeshHandle>) {
        todo!()
    }

    pub fn set_name(&self, _value: Option<String>) {
        todo!()
    }

    pub fn set_rigid_body(&self, _value: JsValue) {
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
