use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

#[wasm_bindgen]
impl Runtime {
    pub fn wired_scene_mesh_clone(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_mesh_colors(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_mesh_drop(&self, _handle: u32) {}
    pub fn wired_scene_mesh_id(&self, _handle: u32) -> String {
        String::new()
    }
    pub fn wired_scene_mesh_indices(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_mesh_name(&self, _handle: u32) -> Option<String> {
        None
    }
    pub fn wired_scene_mesh_normals(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_mesh_positions(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_mesh_set_colors(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_mesh_set_indices(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_mesh_set_name(&self, _handle: u32, _value: Option<String>) {}
    pub fn wired_scene_mesh_set_normals(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_mesh_set_positions(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_mesh_set_sync(&self, _handle: u32, _value: bool) {}
    pub fn wired_scene_mesh_set_tangents(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_mesh_set_topology(&self, _handle: u32, _value: String) {}
    pub fn wired_scene_mesh_set_uv0(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_mesh_set_uv1(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_mesh_sync(&self, _handle: u32) -> bool {
        false
    }
    pub fn wired_scene_mesh_tangents(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_mesh_topology(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_mesh_uv0(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_mesh_uv1(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
}
