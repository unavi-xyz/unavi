use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

#[wasm_bindgen]
impl Runtime {
    pub fn wired_scene_material_alpha_cutoff(&self, _handle: u32) -> f32 {
        0.5
    }
    pub fn wired_scene_material_alpha_mode(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_material_base_color(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_material_clone(&self, _handle: u32) -> JsValue {
        JsValue::UNDEFINED
    }
    pub fn wired_scene_material_double_sided(&self, _handle: u32) -> bool {
        false
    }
    pub fn wired_scene_material_drop(&self, _handle: u32) {}
    pub fn wired_scene_material_id(&self, _handle: u32) -> String {
        String::new()
    }
    pub fn wired_scene_material_metallic(&self, _handle: u32) -> f32 {
        0.0
    }
    pub fn wired_scene_material_name(&self, _handle: u32) -> Option<String> {
        None
    }
    pub fn wired_scene_material_roughness(&self, _handle: u32) -> f32 {
        0.5
    }
    pub fn wired_scene_material_set_alpha_cutoff(&self, _handle: u32, _value: f32) {}
    pub fn wired_scene_material_set_alpha_mode(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_material_set_base_color(&self, _handle: u32, _value: JsValue) {}
    pub fn wired_scene_material_set_double_sided(&self, _handle: u32, _value: bool) {}
    pub fn wired_scene_material_set_metallic(&self, _handle: u32, _value: f32) {}
    pub fn wired_scene_material_set_name(&self, _handle: u32, _value: Option<String>) {}
    pub fn wired_scene_material_set_roughness(&self, _handle: u32, _value: f32) {}
    pub fn wired_scene_material_set_sync(&self, _handle: u32, _value: bool) {}
    pub fn wired_scene_material_set_unlit(&self, _handle: u32, _value: bool) {}
    pub fn wired_scene_material_sync(&self, _handle: u32) -> bool {
        false
    }
    pub fn wired_scene_material_unlit(&self, _handle: u32) -> bool {
        false
    }
}
