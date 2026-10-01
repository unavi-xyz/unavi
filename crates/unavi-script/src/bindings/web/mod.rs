#![expect(clippy::future_not_send)]

use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

mod wired;

#[wasm_bindgen(module = "/dist/runtime.js")]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "instantiateScript")]
    async fn js_instantiate_script(bytes: &[u8], name: &str, runtime: Runtime) -> JsValue;
    #[wasm_bindgen(js_name = "scriptInit")]
    async fn js_script_init(instance: &JsValue);
    #[wasm_bindgen(js_name = "scriptUpdate")]
    async fn js_script_update(instance: &JsValue);
    #[wasm_bindgen(js_name = "scriptFixedUpdate")]
    async fn js_script_fixed_update(instance: &JsValue);
}

pub struct ScriptInstance {
    instance: JsValue,
}

// Safe: wasm is single-threaded
unsafe impl Send for ScriptInstance {}
unsafe impl Sync for ScriptInstance {}

impl ScriptInstance {
    pub async fn instantiate(bytes: &[u8], name: &str, runtime: Runtime) -> Self {
        let instance = js_instantiate_script(bytes, name, runtime).await;
        Self { instance }
    }

    pub async fn init(&self) {
        js_script_init(&self.instance).await;
    }

    pub async fn update(&self) {
        js_script_update(&self.instance).await;
    }

    pub async fn fixed_update(&self) {
        js_script_fixed_update(&self.instance).await;
    }
}

pub type ScriptCell = Arc<std::sync::Mutex<Option<ScriptInstance>>>;
