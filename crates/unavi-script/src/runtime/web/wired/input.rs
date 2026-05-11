use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared::{
        self, Api,
        wired::input::types::{InputAction, InputDevice},
    },
};

use super::scene::node::NodeHandle;

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
        let _ = shared::wired::input::listener::drop(&self.api, self.rep);
    }
}

#[wasm_bindgen]
impl InputListenerHandle {
    pub fn poll(&self) -> JsValue {
        let Ok(Some(event)) = shared::wired::input::listener::poll(&self.api, self.rep) else {
            return JsValue::NULL;
        };

        let action = match event.action {
            InputAction::GrabDown => "grab-down",
            InputAction::GrabUp => "grab-up",
            InputAction::MenuDown => "menu-down",
            InputAction::MenuUp => "menu-up",
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
    pub fn wired_input_register_input_listener(&self, target: &NodeHandle) -> InputListenerHandle {
        let rep = shared::wired::input::register_input_listener(&self.api, target.rep())
            .unwrap_or(u32::MAX);
        InputListenerHandle::new(rep, Arc::clone(&self.api))
    }

    #[wasm_bindgen(js_name = "wiredInputContextListener")]
    pub fn wired_input_context_listener(&self) -> InputListenerHandle {
        let rep =
            shared::wired::input::register_global_input_listener(&self.api).unwrap_or(u32::MAX);
        InputListenerHandle::new(rep, Arc::clone(&self.api))
    }
}
