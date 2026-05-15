use std::sync::Arc;

use wasm_bindgen::{JsValue, prelude::*};

use crate::runtime::shared::{
    self, Api,
    wired::scene::material::{MaterialAlphaMode, MaterialColor},
};

fn color_to_js(c: MaterialColor) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"r".into(), &c.r.into()).ok();
    js_sys::Reflect::set(&obj, &"g".into(), &c.g.into()).ok();
    js_sys::Reflect::set(&obj, &"b".into(), &c.b.into()).ok();
    js_sys::Reflect::set(&obj, &"a".into(), &c.a.into()).ok();
    obj.into()
}

fn js_to_color(v: &JsValue) -> MaterialColor {
    let get = |k: &str, d: f32| {
        js_sys::Reflect::get(v, &k.into())
            .ok()
            .and_then(|v| v.as_f64())
            .map_or(d, |v| v as f32)
    };
    MaterialColor {
        r: get("r", 1.0),
        g: get("g", 1.0),
        b: get("b", 1.0),
        a: get("a", 1.0),
    }
}

#[wasm_bindgen]
pub struct MaterialHandle {
    rep: u32,
    api: Arc<Api>,
}

impl MaterialHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }

    pub const fn rep(&self) -> u32 {
        self.rep
    }
}

impl Drop for MaterialHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let _ = shared::wired::scene::material::on_drop(&self.api, self.rep);
        }
    }
}

#[wasm_bindgen]
impl MaterialHandle {
    pub fn id(&self) -> String {
        shared::wired::scene::material::id(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "__rep", getter)]
    #[expect(clippy::missing_const_for_fn)]
    pub fn wasm_rep(&self) -> u32 {
        self.rep
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_mat(&self) -> Self {
        let rep = shared::wired::scene::material::clone(&self.api, self.rep).unwrap_or(u32::MAX);
        Self::new(rep, Arc::clone(&self.api))
    }

    pub fn name(&self) -> Option<String> {
        shared::wired::scene::material::name(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub fn set_name(&self, value: Option<String>) -> Result<(), String> {
        shared::wired::scene::material::set_name(&self.api, self.rep, value)
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "alphaCutoff")]
    pub fn alpha_cutoff(&self) -> f32 {
        shared::wired::scene::material::alpha_cutoff(&self.api, self.rep).unwrap_or(0.5)
    }

    #[wasm_bindgen(js_name = "setAlphaCutoff")]
    pub fn set_alpha_cutoff(&self, value: f32) -> Result<(), String> {
        shared::wired::scene::material::set_alpha_cutoff(&self.api, self.rep, value)
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "alphaMode")]
    pub fn alpha_mode(&self) -> Option<String> {
        shared::wired::scene::material::alpha_mode(&self.api, self.rep)
            .ok()
            .flatten()
            .map(|m| {
                match m {
                    MaterialAlphaMode::Add => "add",
                    MaterialAlphaMode::Blend => "blend",
                    MaterialAlphaMode::Mask => "mask",
                    MaterialAlphaMode::Multiply => "multiply",
                    MaterialAlphaMode::Opaque => "opaque",
                    MaterialAlphaMode::PreMultiplied => "pre-multiplied",
                }
                .into()
            })
    }

    #[wasm_bindgen(js_name = "setAlphaMode")]
    pub fn set_alpha_mode(&self, value: Option<String>) -> Result<(), String> {
        let mode = value.and_then(|s| match s.as_str() {
            "add" => Some(MaterialAlphaMode::Add),
            "blend" => Some(MaterialAlphaMode::Blend),
            "mask" => Some(MaterialAlphaMode::Mask),
            "multiply" => Some(MaterialAlphaMode::Multiply),
            "opaque" => Some(MaterialAlphaMode::Opaque),
            "pre-multiplied" => Some(MaterialAlphaMode::PreMultiplied),
            _ => None,
        });
        shared::wired::scene::material::set_alpha_mode(&self.api, self.rep, mode)
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "baseColor")]
    pub fn base_color(&self) -> JsValue {
        let c = shared::wired::scene::material::base_color(&self.api, self.rep).unwrap_or(
            MaterialColor {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        );
        color_to_js(c)
    }

    #[wasm_bindgen(js_name = "setBaseColor")]
    pub fn set_base_color(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::material::set_base_color(&self.api, self.rep, js_to_color(&value))
            .map_err(|e| e.to_string())
    }

    pub fn metallic(&self) -> f32 {
        shared::wired::scene::material::metallic(&self.api, self.rep).unwrap_or(0.0)
    }

    #[wasm_bindgen(js_name = "setMetallic")]
    pub fn set_metallic(&self, value: f32) -> Result<(), String> {
        shared::wired::scene::material::set_metallic(&self.api, self.rep, value)
            .map_err(|e| e.to_string())
    }

    pub fn roughness(&self) -> f32 {
        shared::wired::scene::material::roughness(&self.api, self.rep).unwrap_or(0.5)
    }

    #[wasm_bindgen(js_name = "setRoughness")]
    pub fn set_roughness(&self, value: f32) -> Result<(), String> {
        shared::wired::scene::material::set_roughness(&self.api, self.rep, value)
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "doubleSided")]
    pub fn double_sided(&self) -> bool {
        shared::wired::scene::material::double_sided(&self.api, self.rep).unwrap_or(false)
    }

    #[wasm_bindgen(js_name = "setDoubleSided")]
    pub fn set_double_sided(&self, value: bool) -> Result<(), String> {
        shared::wired::scene::material::set_double_sided(&self.api, self.rep, value)
            .map_err(|e| e.to_string())
    }

    pub fn unlit(&self) -> bool {
        shared::wired::scene::material::unlit(&self.api, self.rep).unwrap_or(false)
    }

    #[wasm_bindgen(js_name = "setUnlit")]
    pub fn set_unlit(&self, value: bool) -> Result<(), String> {
        shared::wired::scene::material::set_unlit(&self.api, self.rep, value)
            .map_err(|e| e.to_string())
    }
}
