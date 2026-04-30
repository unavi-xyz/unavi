use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;

use crate::runtime::shared::wired::scene::WiredSceneBackend;

#[wasm_bindgen]
pub struct NodeHandle {
    rep: u32,
    backend: Arc<Mutex<WiredSceneBackend>>,
}

impl NodeHandle {
    pub fn new(rep: u32, backend: Arc<Mutex<WiredSceneBackend>>) -> Self {
        Self { rep, backend }
    }
}

#[wasm_bindgen]
impl NodeHandle {
    pub fn add_child(&self, _child: JsValue) {}

    pub fn children(&self) -> JsValue {
        JsValue::from_str("[]")
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_node(&self) -> JsValue {
        todo!()
    }

    pub fn collider(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn global_transform(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn id(&self) -> String {
        self.backend
            .lock()
            .expect("lock")
            .nodes
            .get(self.rep)
            .map(|n| n.id.to_string())
            .unwrap_or_default()
    }

    pub fn material(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn mesh(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn name(&self) -> Option<String> {
        None
    }

    pub fn parent(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn remove_child(&self, _child: JsValue) {}

    pub fn rigid_body(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn rotation(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn scale(&self) -> JsValue {
        JsValue::UNDEFINED
    }

    pub fn set_collider(&self, _value: JsValue) {}
    pub fn set_material(&self, _value: JsValue) {}
    pub fn set_mesh(&self, _value: JsValue) {}
    pub fn set_name(&self, _value: Option<String>) {}
    pub fn set_rigid_body(&self, _value: JsValue) {}
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
