use std::sync::{Arc, Mutex};

use bevy::prelude::{Entity, Name, World};
use bevy_hsd::cache::{SceneRegistry, SceneRegistryInner};
use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};

use super::state::{DocEntry, NodeEntry};
use super::with_script;
use crate::util::gen_id;

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostSceneContextSelfNode",
        dyn Fn(u32) -> JsValue,
        |id: u32| {
            with_script(id, |state| {
                let inner = state
                    .registry
                    .node_map
                    .lock()
                    .ok()?
                    .get(&state.self_node_id)
                    .cloned()?;
                let rep = state.alloc();
                state.nodes.insert(
                    rep,
                    NodeEntry {
                        inner,
                        doc_entity: state.doc_entity,
                    },
                );
                Some(JsValue::from_f64(f64::from(rep)))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneContextSelfDocument",
        dyn Fn(u32) -> u32,
        |id: u32| {
            with_script(id, |state| {
                let rep = state.alloc();
                state.docs.insert(
                    rep,
                    DocEntry {
                        id: state.doc_id,
                        registry: Arc::clone(&state.registry),
                        doc_entity: state.doc_entity,
                        entity_slot: None,
                        is_public: false,
                        can_read: true,
                        can_write: true,
                    },
                );
                rep
            })
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostSceneContextGetDocument",
        dyn Fn(u32, JsValue) -> JsValue,
        |id: u32, id_bytes: JsValue| {
            let bytes = id_bytes.dyn_ref::<js_sys::Uint8Array>().map(|a| a.to_vec());
            let hash = bytes
                .as_deref()
                .and_then(|b| <&[u8] as TryInto<[u8; 32]>>::try_into(b).ok())
                .map(blake3::Hash::from);
            let hash = match hash {
                Some(h) => h,
                None => return JsValue::NULL,
            };
            with_script(id, |state| {
                if hash == state.doc_id {
                    let rep = state.alloc();
                    state.docs.insert(
                        rep,
                        DocEntry {
                            id: state.doc_id,
                            registry: Arc::clone(&state.registry),
                            doc_entity: state.doc_entity,
                            entity_slot: None,
                            is_public: false,
                            can_read: true,
                            can_write: true,
                        },
                    );
                    Some(JsValue::from_f64(f64::from(rep)))
                } else {
                    None
                }
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneContextCreateDocument",
        dyn Fn(u32) -> JsValue,
        |id: u32| {
            with_script(id, |state| {
                if !state.can_create_document {
                    return None;
                }
                let new_registry = SceneRegistryInner::new();
                let entity_slot: Arc<Mutex<Option<Entity>>> = Arc::new(Mutex::new(None));
                let slot_clone = Arc::clone(&entity_slot);
                let registry_spawn = Arc::clone(&new_registry);
                let doc_id = blake3::hash(gen_id().as_bytes());

                state.command_queue.push(move |world: &mut World| {
                    let entity = world
                        .spawn((
                            SceneRegistry(registry_spawn),
                            Name::new(format!("WebScriptDoc_{}", doc_id)),
                        ))
                        .id();
                    *slot_clone.lock().expect("entity slot lock") = Some(entity);
                });

                let rep = state.alloc();
                state.docs.insert(
                    rep,
                    DocEntry {
                        id: doc_id,
                        registry: new_registry,
                        doc_entity: Entity::PLACEHOLDER,
                        entity_slot: Some(entity_slot),
                        is_public: false,
                        can_read: true,
                        can_write: true,
                    },
                );
                Some(JsValue::from_f64(f64::from(rep)))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneContextRemoveDocument",
        dyn Fn(u32, JsValue),
        |id: u32, id_bytes: JsValue| {
            let bytes = id_bytes.dyn_ref::<js_sys::Uint8Array>().map(|a| a.to_vec());
            let hash = bytes
                .as_deref()
                .and_then(|b| <&[u8] as TryInto<[u8; 32]>>::try_into(b).ok())
                .map(blake3::Hash::from);
            let hash = match hash {
                Some(h) => h,
                None => return,
            };
            with_script(id, |state| {
                let rep = state
                    .docs
                    .iter()
                    .find(|(_, e)| e.id == hash && e.can_write)
                    .map(|(r, _)| *r)?;
                let entry = state.docs.remove(&rep)?;
                let entity = entry
                    .entity_slot
                    .as_ref()
                    .and_then(|s| *s.lock().ok()?)
                    .unwrap_or(entry.doc_entity);
                state.command_queue.push(move |world: &mut World| {
                    world.entity_mut(entity).despawn();
                });
                Some(())
            });
        }
    );
}
