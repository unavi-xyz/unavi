use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::shared::{self, Api};

use super::{material::MaterialHandle, mesh::MeshHandle, node::NodeHandle};

#[wasm_bindgen]
pub struct DocHandle {
    rep: u32,
    api: Arc<Api>,
}

impl DocHandle {
    pub fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

#[wasm_bindgen]
impl DocHandle {
    pub fn id(&self) -> Vec<u8> {
        shared::wired::scene::document::id(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_doc(&self) -> Option<DocHandle> {
        let rep = shared::wired::scene::document::clone(&self.api, self.rep).ok()?;
        Some(DocHandle::new(rep, self.api.clone()))
    }

    pub fn assets(&self) -> js_sys::Array {
        js_sys::Array::new()
    }

    pub fn add_asset(&self, _name: String, _blob_id: Vec<u8>) {}

    pub fn remove_asset(&self, _name: String) {}

    pub fn roots(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::roots(&self.api, self.rep) else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(NodeHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    pub fn nodes(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::nodes(&self.api, self.rep) else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(NodeHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    pub fn create_node(&self) -> NodeHandle {
        let Ok(rep) = shared::wired::scene::document::create_node(&self.api, self.rep) else {
            return NodeHandle::new(u32::MAX, Arc::clone(&self.api));
        };
        NodeHandle::new(rep, Arc::clone(&self.api))
    }

    pub fn remove_node(&self, value: NodeHandle) {
        let _ = shared::wired::scene::document::remove_node(&self.api, value.rep());
    }

    pub fn meshes(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::meshes(&self.api, self.rep) else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(MeshHandle::new(rep)))
            .collect()
    }

    pub fn create_mesh(&self) -> MeshHandle {
        let rep =
            shared::wired::scene::document::create_mesh(&self.api, self.rep).unwrap_or(u32::MAX);
        MeshHandle::new(rep)
    }

    pub fn remove_mesh(&self, value: MeshHandle) {
        let _ = shared::wired::scene::document::remove_mesh(&self.api, value.rep());
    }

    pub fn materials(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::materials(&self.api, self.rep) else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(MaterialHandle::new(rep)))
            .collect()
    }

    pub fn create_material(&self) -> MaterialHandle {
        let rep = shared::wired::scene::document::create_material(&self.api, self.rep)
            .unwrap_or(u32::MAX);
        MaterialHandle::new(rep)
    }

    pub fn remove_material(&self, value: MaterialHandle) {
        let _ = shared::wired::scene::document::remove_material(&self.api, value.rep());
    }
}
