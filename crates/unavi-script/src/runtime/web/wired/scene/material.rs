use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MaterialHandle {
    rep: u32,
}

impl MaterialHandle {
    pub const fn new(rep: u32) -> Self {
        Self { rep }
    }

    pub fn rep(&self) -> u32 {
        self.rep
    }
}

#[wasm_bindgen]
impl MaterialHandle {
    pub fn id(&self) -> String {
        todo!()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_mat(&self) -> Self {
        todo!()
    }

    pub fn name(&self) -> Option<String> {
        todo!()
    }

    pub fn set_name(&self, _value: Option<String>) {
        todo!()
    }

    pub fn alpha_cutoff(&self) -> f32 {
        todo!()
    }

    pub fn set_alpha_cutoff(&self, _value: f32) {
        todo!()
    }

    pub fn alpha_mode(&self) -> JsValue {
        todo!()
    }

    pub fn set_alpha_mode(&self, _value: JsValue) {
        todo!()
    }

    pub fn base_color(&self) -> JsValue {
        todo!()
    }

    pub fn set_base_color(&self, _value: JsValue) {
        todo!()
    }

    pub fn metallic(&self) -> f32 {
        todo!()
    }

    pub fn set_metallic(&self, _value: f32) {
        todo!()
    }

    pub fn roughness(&self) -> f32 {
        todo!()
    }

    pub fn set_roughness(&self, _value: f32) {
        todo!()
    }

    pub fn double_sided(&self) -> bool {
        todo!()
    }

    pub fn set_double_sided(&self, _value: bool) {
        todo!()
    }

    pub fn unlit(&self) -> bool {
        todo!()
    }

    pub fn set_unlit(&self, _value: bool) {
        todo!()
    }

    pub fn sync(&self) -> bool {
        todo!()
    }

    pub fn set_sync(&self, _value: bool) {
        todo!()
    }
}
