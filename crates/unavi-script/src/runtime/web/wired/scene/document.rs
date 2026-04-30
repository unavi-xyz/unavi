use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;

use crate::runtime::shared::wired::scene::WiredSceneBackend;

#[wasm_bindgen]
pub struct DocHandle {
    rep: u32,
    backend: Arc<Mutex<WiredSceneBackend>>,
}

impl DocHandle {
    pub fn new(rep: u32, backend: Arc<Mutex<WiredSceneBackend>>) -> Self {
        Self { rep, backend }
    }
}

#[wasm_bindgen]
impl DocHandle {
    pub fn add_asset(&self, _name: String, _blob_id: Vec<u8>) {}

    pub fn assets(&self) -> JsValue {
        JsValue::from_str("[]")
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_doc(&self) -> JsValue {
        todo!()
    }

    pub fn create_material(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn create_mesh(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn create_node(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn global_transform(&self) -> JsValue {
        JsValue::UNDEFINED
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
        JsValue::from_str("[]")
    }

    pub fn meshes(&self) -> JsValue {
        JsValue::from_str("[]")
    }

    pub fn nodes(&self) -> JsValue {
        JsValue::from_str("[]")
    }

    pub fn public(&self) -> bool {
        false
    }

    pub fn remove_asset(&self, _name: String) {}
    pub fn remove_material(&self, _value: JsValue) {}
    pub fn remove_mesh(&self, _value: JsValue) {}
    pub fn remove_node(&self, _value: JsValue) {}

    pub fn roots(&self) -> JsValue {
        JsValue::from_str("[]")
    }

    pub fn rotation(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn scale(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn set_public(&self, _value: bool) {}
    pub fn set_rotation(&self, _value: JsValue) {}
    pub fn set_scale(&self, _value: JsValue) {}
    pub fn set_sync(&self, _value: bool) {}
    pub fn set_transform(&self, _value: JsValue) {}
    pub fn set_translation(&self, _value: JsValue) {}

    pub fn sync(&self) -> bool {
        false
    }

    pub fn transform(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn translation(&self) -> JsValue {
        JsValue::UNDEFINED
    }
}
