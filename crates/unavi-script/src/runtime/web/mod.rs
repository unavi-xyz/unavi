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
    #[wasm_bindgen(js_name = "scriptRender")]
    async fn js_script_render(instance: &JsValue);
    #[wasm_bindgen(js_name = "scriptTick")]
    async fn js_script_tick(instance: &JsValue);
}

pub struct ScriptInstance {
    instance: JsValue,
    runtime: Runtime,
}

// Safe: wasm is single-threaded
unsafe impl Send for ScriptInstance {}
unsafe impl Sync for ScriptInstance {}

impl ScriptInstance {
    pub async fn instantiate(bytes: &[u8], name: &str, runtime: Runtime) -> Self {
        let instance = js_instantiate_script(bytes, name, runtime.clone()).await;
        Self { instance, runtime }
    }

    pub async fn init(&self) {
        js_script_init(&self.instance).await;
        self.runtime.api.doc.commit();
    }

    pub async fn render(&self) {
        js_script_render(&self.instance).await;
        self.runtime.api.doc.commit();
    }

    pub async fn tick(&self) {
        js_script_tick(&self.instance).await;
        self.runtime.api.doc.commit();
    }
}

pub type ScriptCell = Arc<std::sync::Mutex<Option<ScriptInstance>>>;
