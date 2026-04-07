use bevy::prelude::Entity;
use js_sys::{Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

use super::js_convert::{js_string_array, parse_js_doc_list};
use super::state::WebScriptState;
use super::with_script;
use crate::event_registry::PendingEmission;

fn parse_js_event_filter(state: &WebScriptState, filter: &JsValue) -> (Option<Entity>, f32) {
    let node_rep = Reflect::get(filter, &"nodeRep".into())
        .ok()
        .and_then(|val| val.as_f64())
        .map(|num| num as u32);
    let radius = Reflect::get(filter, &"radius".into())
        .ok()
        .and_then(|val| val.as_f64())
        .unwrap_or(0.0) as f32;
    let entity = node_rep.and_then(|rep| {
        let entry = state.nodes.get(&rep)?;
        *entry.inner.entity.lock().ok()?
    });
    (entity, radius)
}

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostEventEmit",
        dyn Fn(u32, JsValue, JsValue, JsValue),
        |id: u32, channel: JsValue, payload: JsValue, filter: JsValue| {
            with_script(id, |state| {
                let channel = channel.as_string()?;
                let payload = payload
                    .dyn_ref::<js_sys::Uint8Array>()
                    .map(|a| a.to_vec())
                    .unwrap_or_default();
                let (node_entity, radius) = parse_js_event_filter(state, &filter);
                let target_documents =
                    parse_js_doc_list(&Reflect::get(&filter, &"documents".into()).ok()?);
                state
                    .event_registry
                    .0
                    .lock()
                    .ok()?
                    .push_emission(PendingEmission {
                        node: node_entity,
                        channel,
                        payload,
                        radius,
                        sender_doc_id: state.doc_id,
                        target_documents,
                    });
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostEventListen",
        dyn Fn(u32, JsValue, JsValue) -> u32,
        |id: u32, channels: JsValue, filter: JsValue| {
            with_script(id, |state| {
                let channels = js_string_array(&channels)?;
                let (node_entity, radius) = parse_js_event_filter(state, &filter);
                let source_documents =
                    parse_js_doc_list(&Reflect::get(&filter, &"documents".into()).ok()?);
                let doc_id = state.doc_id;
                let queue = {
                    let mut inner = state.event_registry.0.lock().ok()?;
                    if let Some(entity) = node_entity {
                        inner.register_node(entity, channels, radius, source_documents, doc_id)
                    } else {
                        inner.register_global(channels, source_documents, doc_id)
                    }
                };
                let rep = state.alloc();
                state.receptors.insert(rep, queue);
                Some(rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostEventReceptorPoll",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let queue = state.receptors.get(&rep)?;
                let event = queue.lock().ok()?.pop_front()?;
                let obj = Object::new();
                Reflect::set(&obj, &"channel".into(), &JsValue::from_str(&event.channel)).ok();
                let payload = js_sys::Uint8Array::from(event.payload.as_slice());
                Reflect::set(&obj, &"payload".into(), &payload).ok();
                let sender = if event.sender_node.is_some() {
                    "spatial"
                } else {
                    "global"
                };
                Reflect::set(&obj, &"sender".into(), &JsValue::from_str(sender)).ok();
                let sender_doc =
                    js_sys::Uint8Array::from(event.sender_document.as_bytes().as_slice());
                Reflect::set(&obj, &"senderDocument".into(), &sender_doc).ok();
                Reflect::set(&obj, &"time".into(), &JsValue::from_f64(event.time as f64)).ok();
                Some(JsValue::from(obj))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostEventReceptorDrop",
        dyn Fn(u32, u32),
        |id: u32, rep: u32| {
            with_script(id, |state| {
                if let Some(queue) = state.receptors.remove(&rep) {
                    state.event_registry.0.lock().ok()?.remove_receptor(&queue);
                }
                Some(())
            });
        }
    );
}
