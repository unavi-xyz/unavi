use std::sync::Arc;

use bevy::math::Vec3;
use unavi_input::pointer::PointerKind;
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
                Hit,
                InputAction,
                InputEvent,
                Pointer,
                PointerId,
                Ray,
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

/// Holds the claim for as long as the script holds this. Dropping it in JS
/// gives the pointer back, the same as dropping the resource does natively.
#[wasm_bindgen]
pub struct PointerClaimHandle {
    rep: u32,
}

#[wasm_bindgen]
impl PointerClaimHandle {
    #[must_use]
    pub fn id(&self) -> JsValue {
        shared::wired::input::claimed_kind(self.rep).map_or(JsValue::UNDEFINED, |kind| {
            pointer_id_name(kind.into()).into()
        })
    }
}

impl Drop for PointerClaimHandle {
    fn drop(&mut self) {
        shared::wired::input::release_pointer(self.rep);
    }
}

const fn pointer_id_name(id: PointerId) -> &'static str {
    match id {
        PointerId::Screen => "screen",
        PointerId::LeftHand => "left-hand",
        PointerId::RightHand => "right-hand",
    }
}

fn set(object: &js_sys::Object, key: &str, value: &JsValue) {
    js_sys::Reflect::set(object, &key.into(), value).expect("reflect");
}

fn vec2(x: f32, y: f32) -> JsValue {
    let object = js_sys::Object::new();
    set(&object, "x", &x.into());
    set(&object, "y", &y.into());
    object.into()
}

fn vec3(v: Vec3) -> JsValue {
    let object = js_sys::Object::new();
    set(&object, "x", &v.x.into());
    set(&object, "y", &v.y.into());
    set(&object, "z", &v.z.into());
    object.into()
}

fn ray(ray: Ray) -> JsValue {
    let object = js_sys::Object::new();
    set(&object, "origin", &vec3(ray.origin));
    set(&object, "dir", &vec3(ray.dir));
    object.into()
}

fn hit(hit: Option<Hit>) -> JsValue {
    let Some(hit) = hit else {
        return JsValue::UNDEFINED;
    };
    let object = js_sys::Object::new();
    set(&object, "position", &vec3(hit.position));
    set(&object, "normal", &vec3(hit.normal));
    set(&object, "distance", &hit.distance.into());
    object.into()
}

/// A WIT variant crosses into JS as a tag and an optional value, which is how
/// `jco` lowers one.
fn action(action: InputAction) -> JsValue {
    let object = js_sys::Object::new();
    let tag = match action {
        InputAction::Press => "press",
        InputAction::Release => "release",
        InputAction::Scroll(delta) => {
            set(&object, "val", &vec2(delta.x, delta.y));
            "scroll"
        }
        InputAction::Enter => "enter",
        InputAction::Leave => "leave",
        InputAction::MenuPress => "menu-press",
        InputAction::MenuRelease => "menu-release",
    };
    set(&object, "tag", &tag.into());
    object.into()
}

fn event(event: InputEvent) -> JsValue {
    let object = js_sys::Object::new();
    set(&object, "pointer", &pointer_id_name(event.pointer).into());
    set(&object, "action", &action(event.action));
    set(&object, "ray", &ray(event.ray));
    set(&object, "hit", &hit(event.hit));
    object.into()
}

fn pointer(pointer: Pointer) -> JsValue {
    let object = js_sys::Object::new();
    set(&object, "id", &pointer_id_name(pointer.id).into());
    set(&object, "active", &pointer.active.into());
    set(&object, "ray", &ray(pointer.ray));
    set(&object, "grasp", &pointer.grasp.into());
    set(&object, "axis", &vec2(pointer.axis.x, pointer.axis.y));
    set(&object, "hit", &hit(pointer.hit));
    object.into()
}

#[wasm_bindgen]
impl InputListenerHandle {
    pub async fn poll(&self) -> JsValue {
        let Ok(Some(polled)) = shared::wired::input::listener::poll(&self.api, self.rep).await
        else {
            return JsValue::UNDEFINED;
        };
        event(polled)
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredInputListenerClass")]
    #[must_use]
    pub fn wired_input_listener_class(&self) -> JsValue {
        let handle = InputListenerHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &"constructor".into()).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredInputPointerClaimClass")]
    #[must_use]
    pub fn wired_input_pointer_claim_class(&self) -> JsValue {
        let handle = PointerClaimHandle { rep: u32::MAX };
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

    #[wasm_bindgen(js_name = "wiredInputPointers")]
    #[must_use]
    pub fn wired_input_pointers(&self) -> JsValue {
        if self.api.require(ApiName::InputContext).is_err() {
            return js_sys::Array::new().into();
        }
        shared::wired::input::pointers()
            .into_iter()
            .map(pointer)
            .collect::<js_sys::Array>()
            .into()
    }

    #[wasm_bindgen(js_name = "wiredInputClaimPointer")]
    #[must_use]
    pub fn wired_input_claim_pointer(&self, id: &str) -> Option<PointerClaimHandle> {
        if self.api.require(ApiName::InputContext).is_err() {
            return None;
        }
        let kind = match id {
            "screen" => PointerKind::Screen,
            "left-hand" => PointerKind::LeftHand,
            "right-hand" => PointerKind::RightHand,
            _ => return None,
        };
        shared::wired::input::claim_pointer(kind)
            .ok()
            .map(|rep| PointerClaimHandle { rep })
    }
}
