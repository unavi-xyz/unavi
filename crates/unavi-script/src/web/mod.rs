use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_hsd::cache::{NodeHsdChanges, NodeInner, NodeState, SceneRegistry, SceneRegistryInner};
use js_sys::{Array, Function, JsString, Reflect, Uint8Array};
use wasm_bindgen::JsValue;

use bevy_wds::LocalActor;

use crate::asset::Wasm;
use crate::event_registry::{self, EventRegistry};
use crate::input_registry::InputRegistry;
use crate::permissions::{ApiName, ScriptPermissions};
use crate::util::gen_id;

mod host;

pub struct WebScriptPlugin;

impl Plugin for WebScriptPlugin {
    fn build(&self, app: &mut App) {
        host::init();

        app.init_resource::<ScriptIdCounter>()
            .add_systems(
                FixedUpdate,
                (
                    crate::firewall::sync_hsd_firewall_entities,
                    (
                        poll_web_scripts,
                        tick_web_scripts,
                        flush_web_script_commands,
                        event_registry::process_event_emissions,
                    )
                        .chain(),
                )
                    .after(bevy_hsd::hydrate::init::init_hsd_doc),
            )
            .add_systems(Update, render_web_scripts);
    }
}

#[derive(Component)]
pub struct WebPendingScript(pub Handle<Wasm>);

#[derive(Component)]
pub struct WebLoadedScript(pub u32);

#[derive(Resource, Default)]
struct ScriptIdCounter(u32);

fn spawn_self_node(registry: &SceneRegistryInner, self_node_id: &smol_str::SmolStr) {
    let inner = Arc::new(NodeInner {
        entity: Mutex::new(None),
        hsd_changes: Mutex::new(NodeHsdChanges::default()),
        id: self_node_id.clone(),
        is_virtual: false,
        state: Mutex::new(NodeState::default()),
        sync: false.into(),
        tree_id: Mutex::new(None),
    });
    registry
        .node_map
        .lock()
        .expect("node_map lock")
        .insert(self_node_id.clone(), Arc::clone(&inner));
    registry.nodes.lock().expect("nodes lock").push(inner);
}

fn poll_web_scripts(
    mut commands: Commands,
    wasm_assets: Res<Assets<Wasm>>,
    pending: Query<(
        Entity,
        &WebPendingScript,
        Option<&Name>,
        Option<&ScriptPermissions>,
    )>,
    mut counter: ResMut<ScriptIdCounter>,
    event_reg: Res<EventRegistry>,
    input_reg: Res<InputRegistry>,
    actors: Query<&LocalActor>,
) {
    let wds_actor = actors.single().map(|a| a.0.clone()).ok();

    for (entity, pending, name, perms) in &pending {
        let Some(wasm) = wasm_assets.get(&pending.0) else {
            continue;
        };

        let script_id = counter.0;
        counter.0 += 1;

        if let Some(name) = name {
            info!(name = %name, script_id, "loading web script");
        }

        let registry = SceneRegistryInner::new();
        let self_node_id = gen_id();
        spawn_self_node(&registry, &self_node_id);
        let camera_node_id = gen_id();
        spawn_self_node(&registry, &camera_node_id);

        let doc_entity = commands
            .spawn((
                SceneRegistry(Arc::clone(&registry)),
                Name::new(format!("WebScriptDoc_{script_id}")),
            ))
            .id();

        let doc_id = blake3::hash(&script_id.to_le_bytes());

        let can_create_document = perms
            .map(|p| p.api.contains(&ApiName::CreateDocument))
            .unwrap_or(false);

        host::register_script(
            script_id,
            host::new_script_state(
                registry,
                doc_entity,
                doc_id,
                self_node_id,
                camera_node_id,
                event_reg.clone(),
                input_reg.clone(),
                wds_actor.clone(),
                can_create_document,
            ),
        );

        let bytes = Uint8Array::from(wasm.0.as_slice());
        call_loader("loadScriptBytes", &[JsValue::from(script_id), bytes.into()]);

        commands
            .entity(entity)
            .remove::<WebPendingScript>()
            .insert(WebLoadedScript(script_id));
    }
}

fn tick_web_scripts(loaded: Query<&WebLoadedScript>) {
    for script in &loaded {
        call_loader("tickScript", &[JsValue::from(script.0)]);
    }
}

fn flush_web_script_commands(mut commands: Commands, loaded: Query<&WebLoadedScript>) {
    for script in &loaded {
        if let Some(mut queue) = host::drain_commands(script.0) {
            commands.append(&mut queue);
        }
    }
}

fn render_web_scripts(loaded: Query<&WebLoadedScript>) {
    for script in &loaded {
        call_loader("renderScript", &[JsValue::from(script.0)]);
    }
}

fn call_loader(fn_name: &str, args: &[JsValue]) {
    let global = js_sys::global();
    let loader = match Reflect::get(&global, &JsString::from("__unavi_loader")) {
        Ok(v) if !v.is_undefined() && !v.is_null() => v,
        _ => {
            error!("__unavi_loader not found on globalThis");
            return;
        }
    };
    let f: Function = match Reflect::get(&loader, &JsString::from(fn_name)) {
        Ok(v) => v.into(),
        Err(_) => {
            error!("__unavi_loader.{fn_name} not found");
            return;
        }
    };
    let arr = Array::new();
    for arg in args {
        arr.push(arg);
    }
    if let Err(e) = f.apply(&loader, &arr) {
        error!("__unavi_loader.{fn_name} error: {e:?}");
    }
}
