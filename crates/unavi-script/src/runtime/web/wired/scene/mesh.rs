use std::sync::Arc;

use tokio::sync::Mutex;
use wasm_bindgen::prelude::*;

use crate::runtime::shared::wired::scene::WiredSceneBackend;

#[wasm_bindgen]
pub struct MeshHandle {
    rep: u32,
    backend: Arc<Mutex<WiredSceneBackend>>,
}

impl MeshHandle {
    pub const fn new(rep: u32, backend: Arc<Mutex<WiredSceneBackend>>) -> Self {
        Self { rep, backend }
    }

    pub fn rep(&self) -> u32 {
        self.rep
    }
}

#[wasm_bindgen]
impl MeshHandle {
    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_mesh(&self) -> Self {
        todo!()
    }

    pub fn colors(&self) -> JsValue {
        todo!()
    }

    pub fn id(&self) -> String {
        todo!()
    }

    pub fn indices(&self) -> JsValue {
        todo!()
    }

    pub fn name(&self) -> Option<String> {
        todo!()
    }

    pub fn normals(&self) -> JsValue {
        todo!()
    }

    pub fn positions(&self) -> JsValue {
        todo!()
    }

    pub fn set_colors(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_indices(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_name(&self, _value: Option<String>) {
        todo!()
    }

    pub fn set_normals(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_positions(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_sync(&self, _value: bool) {
        todo!()
    }

    pub fn set_tangents(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_topology(&self, _value: String) {
        todo!()
    }

    pub fn set_uv0(&self, _value: JsValue) {
        todo!()
    }

    pub fn set_uv1(&self, _value: JsValue) {
        todo!()
    }

    pub fn sync(&self) -> bool {
        todo!()
    }

    pub fn tangents(&self) -> JsValue {
        todo!()
    }

    pub fn topology(&self) -> JsValue {
        todo!()
    }

    pub fn uv0(&self) -> JsValue {
        todo!()
    }

    pub fn uv1(&self) -> JsValue {
        todo!()
    }
}
