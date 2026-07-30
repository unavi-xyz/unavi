use std::sync::Arc;

use unavi_util::async_task::spawn_async_task;
use wasm_bindgen::prelude::*;

use super::scene::prim::PrimHandle;
use crate::{
    permissions::ApiName,
    runtime::{
        Runtime,
        shared::{
            self,
            Api,
            wired::input::types::{
                InputAction,
                InputDevice,
            },
        },
    },
};

#[wasm_bindgen]
pub struct InputListenerHandle {
    rep: u32,
    api: Arc<Api>,
}

impl InputListenerHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for InputListenerHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::input::listener::drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
impl InputListenerHandle {
    pub async fn poll(&self) -> JsValue {
        let Ok(Some(event)) = shared::wired::input::listener::poll(&self.api, self.rep).await
        else {
            return JsValue::UNDEFINED;
        };

        let action = match event.action {
            InputAction::GrabDown => "grab-down",
            InputAction::GrabUp => "grab-up",
            InputAction::MenuDown => "menu-down",
            InputAction::MenuUp => "menu-up",
            InputAction::ScrollUp => "scroll-up",
            InputAction::ScrollDown => "scroll-down",
        };
        let device = match event.device {
            InputDevice::Keyboard => "keyboard",
            InputDevice::LeftHand => "left-hand",
            InputDevice::RightHand => "right-hand",
        };

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"action".into(), &action.into()).expect("reflect");
        js_sys::Reflect::set(&obj, &"device".into(), &device.into()).expect("reflect");
        obj.into()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredInputListenerClass")]
    pub fn wired_input_listener_class(&self) -> JsValue {
        let handle = InputListenerHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &"constructor".into()).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredInputRegisterInputListener")]
    pub async fn wired_input_register_input_listener(
        &self,
        target: &PrimHandle,
    ) -> InputListenerHandle {
        let rep = match self.api.require(ApiName::Input) {
            Ok(()) => shared::wired::input::register_input_listener(&self.api, target.rep())
                .await
                .unwrap_or(u32::MAX),
            Err(_) => u32::MAX,
        };
        InputListenerHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "wiredInputRegisterGlobalInputListener")]
    pub async fn wired_input_register_global_input_listener(&self) -> InputListenerHandle {
        let rep = match self.api.require(ApiName::InputContext) {
            Ok(()) => shared::wired::input::register_global_input_listener(&self.api)
                .await
                .unwrap_or(u32::MAX),
            Err(_) => u32::MAX,
        };
        InputListenerHandle::new(rep, Arc::clone(&self.api))
    }
}
