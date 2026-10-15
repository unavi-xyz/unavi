//! `wired:input/types`, `wired:input/targeted` and `wired:input/device`.

use std::rc::Rc;

use unavi_input::pointer::PointerKind;
use wasm_bindgen::prelude::*;

use super::{
    HostHandle,
    Runtime,
    convert,
    scene::DocumentHandle,
};
use crate::host::input::{
    self,
    Button,
};

/// A guest's open input subscription. See `MessageSubscriptionHandle` in
/// `bindings::web::event` for how its `Drop` comes to run.
#[wasm_bindgen]
pub struct InputSubscriptionHandle {
    rep:  u32,
    host: HostHandle,
}

impl Drop for InputSubscriptionHandle {
    fn drop(&mut self) {
        let _ = input::close(&mut self.host.borrow_mut(), self.rep);
    }
}

const fn wit_pointer_kind(kind: PointerKind) -> &'static str {
    match kind {
        PointerKind::Screen => "screen",
        PointerKind::LeftHand => "left-hand",
        PointerKind::RightHand => "right-hand",
    }
}

const fn wit_button(button: Button) -> &'static str {
    match button {
        Button::Trigger => "trigger",
        Button::Grip => "grip",
        Button::Menu => "menu",
    }
}

fn wit_hit(hit: input::Hit) -> JsValue {
    let obj = js_sys::Object::new();
    let set = |key: &str, value: &JsValue| {
        js_sys::Reflect::set(&obj, &JsValue::from_str(key), value).ok();
    };
    set(
        "target",
        &hit.target.map_or(JsValue::UNDEFINED, convert::wit_prim_ref),
    );
    set("position", &convert::wit_vec3(hit.position));
    set("normal", &convert::wit_vec3(hit.normal));
    set("distance", &JsValue::from_f64(f64::from(hit.distance)));
    obj.into()
}

fn tagged(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("tag"), &JsValue::from_str(tag)).ok();
    if !val.is_undefined() {
        js_sys::Reflect::set(&obj, &JsValue::from_str("val"), &val).ok();
    }
    obj.into()
}

fn wit_action(action: input::Action) -> JsValue {
    match action {
        input::Action::Pressed(b) => tagged("pressed", JsValue::from_str(wit_button(b))),
        input::Action::Released(b) => tagged("released", JsValue::from_str(wit_button(b))),
        input::Action::Scroll(delta) => tagged("scroll", convert::wit_vec2(delta)),
        input::Action::Entered => tagged("entered", JsValue::UNDEFINED),
        input::Action::Left => tagged("left", JsValue::UNDEFINED),
    }
}

fn wit_event(event: input::InputEvent) -> JsValue {
    let obj = js_sys::Object::new();
    let set = |key: &str, value: &JsValue| {
        js_sys::Reflect::set(&obj, &JsValue::from_str(key), value).ok();
    };
    set(
        "pointer",
        &JsValue::from_str(wit_pointer_kind(event.pointer)),
    );
    set("action", &wit_action(event.action));
    set(
        "ray",
        &convert::wit_ray(event.ray.origin, event.ray.direction),
    );
    set("hit", &event.hit.map_or(JsValue::UNDEFINED, wit_hit));
    obj.into()
}

fn wit_pointer(pointer: input::Pointer) -> JsValue {
    let obj = js_sys::Object::new();
    let set = |key: &str, value: &JsValue| {
        js_sys::Reflect::set(&obj, &JsValue::from_str(key), value).ok();
    };
    set("kind", &JsValue::from_str(wit_pointer_kind(pointer.kind)));
    set("active", &JsValue::from_bool(pointer.active));
    set(
        "ray",
        &convert::wit_ray(pointer.ray.origin, pointer.ray.direction),
    );
    set("trigger", &JsValue::from_f64(f64::from(pointer.trigger)));
    set("grip", &JsValue::from_f64(f64::from(pointer.grip)));
    set("axis", &convert::wit_vec2(pointer.axis));
    set("hit", &pointer.hit.map_or(JsValue::UNDEFINED, wit_hit));
    obj.into()
}

#[wasm_bindgen]
impl InputSubscriptionHandle {
    pub fn drain(&self, max: u32) -> Result<Vec<JsValue>, JsValue> {
        input::drain(&self.host.borrow(), self.rep, max)
            .map(|events| events.into_iter().map(wit_event).collect())
            .map_err(convert::into_trap)
    }

    pub fn dropped(&self) -> Result<u64, JsValue> {
        input::dropped(&self.host.borrow(), self.rep).map_err(convert::into_trap)
    }
}

#[wasm_bindgen]
impl Runtime {
    /// `wired:input/targeted.listen`.
    #[wasm_bindgen(js_name = "inputTargetedListen")]
    pub fn input_targeted_listen(
        &self,
        document: &DocumentHandle,
        prim: JsValue,
    ) -> Result<InputSubscriptionHandle, JsValue> {
        let prim = convert::prim_id(prim).map_err(convert::raise)?;
        let rep = input::listen_targeted(&mut self.host.borrow_mut(), document.rep(), prim)
            .map_err(convert::raise)?;
        Ok(InputSubscriptionHandle {
            rep,
            host: Rc::clone(&self.host),
        })
    }

    /// `wired:input/device.listen`.
    #[wasm_bindgen(js_name = "inputDeviceListen")]
    pub fn input_device_listen(&self) -> Result<InputSubscriptionHandle, JsValue> {
        let rep = input::listen_device(&mut self.host.borrow_mut()).map_err(convert::raise)?;
        Ok(InputSubscriptionHandle {
            rep,
            host: Rc::clone(&self.host),
        })
    }

    /// `wired:input/device.pointers`.
    #[wasm_bindgen(js_name = "inputDevicePointers")]
    pub fn input_device_pointers(&self) -> Result<Vec<JsValue>, JsValue> {
        input::pointers(&self.host.borrow())
            .map(|pointers| pointers.into_iter().map(wit_pointer).collect())
            .map_err(convert::raise)
    }
}
