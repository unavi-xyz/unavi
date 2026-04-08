use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use bevy::ecs::world::CommandQueue;
use bevy_hsd::cache::SceneRegistryInner;
use bevy_hsd::hydrate::events::ScriptCommandQueue;
use js_sys::{Object, Reflect};
use smol_str::SmolStr;
use std::sync::Arc;
use wasm_bindgen::JsValue;

use crate::event_registry::EventRegistry;
use crate::input_registry::InputRegistry;

use self::state::WebScriptState;

macro_rules! reg {
    ($obj:expr, $name:literal, $t:ty, $body:expr) => {{
        let c: wasm_bindgen::closure::Closure<$t> =
            wasm_bindgen::closure::Closure::wrap(Box::new($body));
        js_sys::Reflect::set(
            $obj,
            &wasm_bindgen::JsValue::from_str($name),
            c.as_ref().unchecked_ref(),
        )
        .unwrap();
        c.forget();
    }};
}

mod agent;
mod document;
mod event;
mod input;
pub mod js_convert;
mod material;
mod mesh;
mod node;
mod scene_context;
pub mod state;
mod wds;

struct WebHostState {
    scripts: HashMap<u32, WebScriptState>,
}

static WEB_HOST: OnceLock<Mutex<WebHostState>> = OnceLock::new();

pub(super) fn with_script<T>(id: u32, f: impl FnOnce(&mut WebScriptState) -> T) -> Option<T> {
    let host = WEB_HOST.get()?;
    let mut guard = host.lock().ok()?;
    let state = guard.scripts.get_mut(&id)?;
    Some(f(state))
}

pub(super) fn register_script(id: u32, state: WebScriptState) {
    if let Some(host) = WEB_HOST.get() {
        if let Ok(mut guard) = host.lock() {
            guard.scripts.insert(id, state);
        }
    }
}

pub(super) fn drain_commands(id: u32) -> Option<CommandQueue> {
    let host = WEB_HOST.get()?;
    let mut guard = host.lock().ok()?;
    let state = guard.scripts.get_mut(&id)?;
    if state.command_queue.len == 0 {
        return None;
    }
    let mut empty = ScriptCommandQueue::default();
    std::mem::swap(&mut state.command_queue, &mut empty);
    Some(empty.inner)
}

pub(super) fn new_script_state(
    registry: Arc<SceneRegistryInner>,
    doc_entity: bevy::prelude::Entity,
    doc_id: blake3::Hash,
    self_node_id: SmolStr,
    camera_node_id: SmolStr,
    event_registry: EventRegistry,
    input_registry: InputRegistry,
    wds_actor: Option<::wds::actor::Actor>,
) -> WebScriptState {
    WebScriptState {
        registry,
        command_queue: ScriptCommandQueue::default(),
        doc_entity,
        doc_id,
        self_node_id,
        camera_node_id,
        event_registry,
        input_registry,
        wds_actor,
        next_rep: 0,
        nodes: HashMap::new(),
        docs: HashMap::new(),
        meshes: HashMap::new(),
        mats: HashMap::new(),
        receptors: HashMap::new(),
        listeners: HashMap::new(),
        wds_instances: HashMap::new(),
        wds_query_futures: HashMap::new(),
        wds_read_futures: HashMap::new(),
    }
}

pub(super) fn init() {
    WEB_HOST.get_or_init(|| {
        Mutex::new(WebHostState {
            scripts: HashMap::new(),
        })
    });

    let global = js_sys::global();
    let obj = Object::new();

    agent::register(&obj);
    document::register(&obj);
    event::register(&obj);
    input::register(&obj);
    material::register(&obj);
    mesh::register(&obj);
    node::register(&obj);
    scene_context::register(&obj);
    wds::register(&obj);

    Reflect::set(&global, &JsValue::from_str("__unavi_host"), &obj).unwrap();
}
