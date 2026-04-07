use bevy::prelude::Entity;
use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};

use super::with_script;
use crate::input_registry::{InputAction, InputDevice};

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostInputRegisterListener",
        dyn Fn(u32, u32) -> u32,
        |id: u32, node_rep: u32| {
            with_script(id, |state| {
                let entity = *state.nodes.get(&node_rep)?.inner.entity.lock().ok()?;
                let entity = entity.unwrap_or(Entity::PLACEHOLDER);
                let queue = state.input_registry.0.lock().ok()?.register_node(entity);
                let rep = state.alloc();
                state.listeners.insert(rep, (queue, Some(entity)));
                Some(rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostInputSystemListener",
        dyn Fn(u32) -> u32,
        |id: u32| {
            with_script(id, |state| {
                let queue = state.input_registry.0.lock().ok()?.register_system();
                let rep = state.alloc();
                state.listeners.insert(rep, (queue, None));
                Some(rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostInputListenerPoll",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let (queue, _) = state.listeners.get(&rep)?;
                let event = queue.lock().ok()?.pop_front()?;
                let obj = Object::new();
                let action = match event.action {
                    InputAction::GrabDown => 0,
                    InputAction::GrabUp => 1,
                    InputAction::MenuDown => 2,
                    InputAction::MenuUp => 3,
                };
                let device = match event.device {
                    InputDevice::Keyboard => 0,
                    InputDevice::LeftHand => 1,
                    InputDevice::RightHand => 2,
                };
                js_sys::Reflect::set(
                    &obj,
                    &"action".into(),
                    &JsValue::from_f64(f64::from(action)),
                )
                .ok();
                js_sys::Reflect::set(
                    &obj,
                    &"device".into(),
                    &JsValue::from_f64(f64::from(device)),
                )
                .ok();
                Some(JsValue::from(obj))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostInputListenerDrop",
        dyn Fn(u32, u32),
        |id: u32, rep: u32| {
            with_script(id, |state| {
                if let Some((queue, entity)) = state.listeners.remove(&rep) {
                    if let Ok(mut inner) = state.input_registry.0.lock() {
                        match entity {
                            Some(entity) => inner.remove_node(entity, &queue),
                            None => inner.remove_system(&queue),
                        }
                    }
                }
                Some(())
            });
        }
    );
}
