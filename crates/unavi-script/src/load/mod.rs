use std::{sync::Arc, sync::Mutex, task::Poll};

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use bevy_hsd::{
    HsdDoc, HsdRecordId,
    cache::{SceneRegistry, SceneRegistryInner},
    hydrate::events::ScriptEventQueue,
};
use log::{ScriptStderr, ScriptStdout};
use loro::{LoroDoc, TreeID};
use smol_str::ToSmolStr;
use state::{RuntimeData, StoreState};
use unavi_agent::LocalAgent;
use wasmtime::{AsContextMut, Store, component::Linker};
use wasmtime_wasi::WasiCtxBuilder;

use bevy_wds::{LocalActor, LocalBlobs};

use crate::{
    ScriptEngine, WasmBinary, WasmEngine,
    agent::NeedsAgentProxy,
    api::wired::scene::document::gen_id,
    asset::Wasm,
    permissions::{ApiName, HsdPermissions, ScriptPermissions},
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

#[expect(clippy::too_many_lines)]
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
    hsd_change_queues: Query<&ScriptEventQueue>,
    permissions: Query<Option<&ScriptPermissions>>,
    local_agent_ent: Query<Entity, With<LocalAgent>>,
    input_registry: Res<crate::api::wired::input::InputRegistry>,
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

        let mut perms = permissions
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

        let mut maybe_agent_ent: Option<Entity> = None;

        let (doc, self_node_id, registry, events, agent_entry, doc_id, doc_entity) =
            if perms.api.contains(&ApiName::LocalAgent) {
                let Ok(agent_ent) = local_agent_ent.single() else {
                    // Wait for local agent to be loaded.
                    continue;
                };
                maybe_agent_ent = Some(agent_ent);

                // Placeholder registry + doc — init_agent_proxies will replace registry.
                let placeholder_registry = SceneRegistryInner::new();
                let self_node_id = gen_id();
                let dummy_doc = Arc::new(LoroDoc::new());
                let doc_id = blake3::hash(&dummy_doc.peer_id().to_le_bytes());
                let agent_events = Arc::new(Mutex::new(Vec::new()));
                let doc_ent = commands
                    .spawn((
                        ScriptEventQueue(Arc::clone(&agent_events)),
                        HsdRecordId(doc_id),
                        SceneRegistry(Arc::clone(&placeholder_registry)),
                    ))
                    .id();

                (
                    dummy_doc,
                    self_node_id,
                    placeholder_registry,
                    agent_events,
                    None,
                    doc_id,
                    doc_ent,
                )
            } else {
                let Ok(registry) = registries.get(source.doc_entity) else {
                    warn!("SceneRegistry not found for script");
                    continue;
                };
                let Ok(event_queue) = hsd_change_queues.get(source.doc_entity) else {
                    warn!("HsdChangeQueue not found for script");
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

                (
                    doc,
                    self_tree_id.to_smolstr(),
                    Arc::clone(&registry.0),
                    Arc::clone(&event_queue.0),
                    None,
                    doc_id,
                    source.doc_entity,
                )
            };

        perms.hsd.insert(
            doc_id,
            [HsdPermissions::Read, HsdPermissions::Write]
                .into_iter()
                .collect(),
        );

        let rt = RuntimeData::new(
            actor.clone(),
            blobs.clone(),
            doc,
            self_node_id,
            registry,
            events,
            perms.clone(),
            agent_entry,
            doc_id,
            doc_entity,
            input_registry.clone(),
        );
        let state = StoreState::new(wasi_ctx, rt);

        let mut store = Store::new(&engine.0, state);
        store.epoch_deadline_async_yield_and_update(1);

        let component = wasmtime::component::Component::from_binary(&engine.0, &wasm.0);

        let rt = ScriptRuntime::new(store, stdout, stderr);
        let ctx = Arc::clone(&rt.ctx);
        commands.entity(ent).insert((LoadingScript, rt));
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
