use std::{
    sync::{Arc, Mutex},
    task::Poll,
};

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use bevy_hsd::{
    HsdDoc, HsdRecordId,
    cache::SceneRegistry,
    hydrate::events::{ScriptCommandQueue, ScriptCommandQueueComp},
};
use bevy_wds::{LocalActor, LocalBlobs};
use log::{ScriptStderr, ScriptStdout};
use loro::{LoroDoc, TreeID};
use smol_str::ToSmolStr;
use state::{RuntimeData, StoreState};
use unavi_agent::LocalAgent;
use wasmtime::{AsContextMut, Store, component::Linker};
use wasmtime_wasi::WasiCtxBuilder;

use crate::{
    EventRegistry, InputRegistry, ScriptEngine, WasmBinary, WasmEngine,
    agent::NeedsAgentProxy,
    api::wired::scene::{DocHandle, GlobalRegistryMapRes},
    asset::Wasm,
    firewall::HsdFirewall,
    permissions::{ApiName, ScriptPermissions},
    runtime::{RuntimeCtx, ScriptRuntime},
};

pub mod hsd;
pub mod local;
pub mod log;
pub mod state;

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-script",
        imports: {
            default: async,
        },
        exports: {
            default: async,
        }
    });
}

type LoadResult = anyhow::Result<(Entity, bindings::Guest)>;

#[derive(Component)]
pub struct LoadingScript;

#[derive(Component)]
#[require(Executing)]
pub struct LoadedScript(pub Arc<bindings::Guest>);

#[derive(Component, Default, Deref, DerefMut)]
pub struct Executing(bool);

pub(crate) fn load_scripts(
    mut commands: Commands,
    wasm_assets: Res<Assets<Wasm>>,
    engines: Query<&WasmEngine>,
    to_load: Query<
        (
            Entity,
            &WasmBinary,
            &ScriptEngine,
            Option<&Name>,
            &hsd::HsdScriptSource,
        ),
        (Without<LoadingScript>, Without<LoadedScript>),
    >,
    mut pool: TaskPool<LoadResult>,
    local_actors: Query<&LocalActor>,
    local_blobs: Query<&LocalBlobs>,
    hsd_docs: Query<&HsdDoc>,
    hsd_record_ids: Query<&HsdRecordId>,
    registries: Query<&SceneRegistry>,
    permissions: Query<Option<&ScriptPermissions>>,
    local_agent_ent: Query<Entity, With<LocalAgent>>,
    input_registry: Res<InputRegistry>,
    event_registry: Res<EventRegistry>,
    registry_map_res: Res<GlobalRegistryMapRes>,
) {
    #[cfg(target_family = "wasm")]
    return;

    let actor = local_actors.single().ok().map(|a| a.0.clone());
    let blobs = local_blobs.single().ok().map(|b| b.0.clone());

    for (ent, handle, script, name, source) in to_load {
        let Ok(engine) = engines.get(script.0) else {
            warn!("Script instantiation failed: engine not found");
            continue;
        };

        let Some(wasm) = wasm_assets.get(&handle.0) else {
            continue;
        };

        let perms = permissions
            .get(source.doc_entity)
            .ok()
            .flatten()
            .cloned()
            .unwrap_or_default();

        let name = name.map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);

        let (stdout, stdout_stream) = ScriptStdout::new();
        let (stderr, stderr_stream) = ScriptStderr::new();
        let wasi_ctx = WasiCtxBuilder::new()
            .stdout(stdout_stream)
            .stderr(stderr_stream)
            .build();

        let mut maybe_agent_ent = None;

        let cmd_queue = Arc::new(Mutex::new(ScriptCommandQueue::default()));

        if perms.api.contains(&ApiName::LocalAgent) {
            let Ok(agent_ent) = local_agent_ent.single() else {
                // Wait for local agent to be loaded.
                continue;
            };
            maybe_agent_ent = Some(agent_ent);
        }
        let Ok(registry) = registries.get(source.doc_entity) else {
            warn!("SceneRegistry not found for script");
            continue;
        };
        let Ok(self_tree_id) = TreeID::try_from(source.node_id.as_str()) else {
            warn!("invalid tree id: {}", source.node_id);
            continue;
        };
        let doc = hsd_docs
            .get(source.doc_entity)
            .map_or_else(|_| Arc::new(LoroDoc::new()), |hsd| Arc::clone(&hsd.0));

        let doc_id = hsd_record_ids
            .get(source.doc_entity)
            .map_or_else(|_| blake3::hash(b"unknown"), |r| r.0);

        let rt = RuntimeData::new(
            actor.clone(),
            blobs.clone(),
            doc,
            self_tree_id.to_smolstr(),
            Arc::clone(&registry.0),
            Arc::clone(&cmd_queue),
            None,
            doc_id,
            source.doc_entity,
            input_registry.clone(),
            event_registry.clone(),
            Arc::clone(&registry_map_res.0),
        );
        let state = StoreState::new(wasi_ctx, rt);

        let mut store = Store::new(&engine.0, state);
        store.epoch_deadline_async_yield_and_update(1);

        let component = wasmtime::component::Component::from_binary(&engine.0, &wasm.0);

        let rt = ScriptRuntime::new(store, stdout, stderr);
        let ctx = Arc::clone(&rt.ctx);
        commands
            .entity(ent)
            .insert((LoadingScript, rt, ScriptCommandQueueComp(cmd_queue)));

        if let Some(agent_ent) = maybe_agent_ent {
            commands.entity(ent).insert(NeedsAgentProxy(agent_ent));
        }

        info!(name, "instantiating script");
        pool.spawn(async move {
            let mut ctx = ctx.lock().await;
            let res = instantiate_component(ent, component, &mut ctx, perms)
                .await
                .with_context(|| name)?;
            drop(ctx);
            Ok(res)
        });
    }

    for task in pool.iter_poll() {
        match task {
            Poll::Ready(Ok((ent, script))) => {
                commands
                    .entity(ent)
                    .remove::<LoadingScript>()
                    .insert(LoadedScript(Arc::new(script)));
            }
            Poll::Ready(Err(e)) => {
                error!("Error instantiating script component: {e:?}");
            }
            _ => {}
        }
    }
}

/// Registers newly created doc entities into the `GlobalRegistryMap` so scripts
/// can access foreign documents via `get-document`.
pub(crate) fn register_new_docs(
    new_docs: Query<
        (Entity, &HsdRecordId, &SceneRegistry, Option<&HsdFirewall>),
        Added<SceneRegistry>,
    >,
    registry_map: Res<GlobalRegistryMapRes>,
    mut commands: Commands,
) {
    for (doc_entity, record_id, registry, firewall) in &new_docs {
        let hsd_fw = firewall.map_or_else(
            || {
                let fw = HsdFirewall::default();
                let inner = Arc::clone(&fw.0);
                commands.entity(doc_entity).insert(fw);
                inner
            },
            |fw| Arc::clone(&fw.0),
        );
        let handle = DocHandle {
            registry: Arc::clone(&registry.0),
            doc_entity,
            firewall: hsd_fw,
        };
        registry_map
            .0
            .write()
            .expect("registry_map write")
            .insert(record_id.0, handle);
    }
}

/// Removes a doc entity's entry from `GlobalRegistryMap` when its `HsdRecordId`
/// is removed (i.e. the entity is despawned).
pub(crate) fn on_hsd_record_removed(
    trigger: On<Remove, HsdRecordId>,
    query: Query<&HsdRecordId>,
    registry_map: Res<GlobalRegistryMapRes>,
) {
    if let Ok(record_id) = query.get(trigger.entity) {
        registry_map
            .0
            .write()
            .expect("registry_map write")
            .remove(&record_id.0);
    }
}

async fn instantiate_component(
    ent: Entity,
    component: Result<wasmtime::component::Component, anyhow::Error>,
    rt: &mut RuntimeCtx,
    perms: ScriptPermissions,
) -> LoadResult {
    let mut linker = Linker::new(rt.store.engine());
    wasmtime_wasi::p2::add_to_linker_async(&mut linker).context("add wasi to linker")?;
    crate::api::wired::add_to_linker(&mut linker, &perms)?;

    let component = component.context("component load")?;

    let guest = bindings::Guest::instantiate_async(rt.store.as_context_mut(), &component, &linker)
        .await
        .context("instantiate guest")?;

    Ok((ent, guest))
}
