use std::{collections::HashMap, sync::Arc, sync::Mutex, task::Poll};

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use bevy_hsd::{
    HsdDoc, HsdRecordId,
    cache::{NodeInner, NodeState, SceneRegistry, SceneRegistryInner},
    data::HsdNodeData,
    hydrate::events::{DocChange, DocChangeKind, DocChangeQueue},
};
use log::{ScriptStderr, ScriptStdout};
use loro::{LoroDoc, TreeID};
use smol_str::ToSmolStr;
use state::{RuntimeData, StoreState};
use wasmtime::{AsContextMut, Store, component::Linker};
use wasmtime_wasi::WasiCtxBuilder;

use bevy_wds::{LocalActor, LocalBlobs};

use crate::{
    ScriptEngine, WasmBinary, WasmEngine,
    agent::{AgentDocEntry, LocalAgentDocs},
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

#[expect(clippy::too_many_arguments)]
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
    hsd_change_queues: Query<&DocChangeQueue>,
    permissions: Query<Option<&ScriptPermissions>>,
    agent_docs: Option<Res<LocalAgentDocs>>,
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
        info!(name, "instantiating script");

        let (stdout, stdout_stream) = ScriptStdout::new();
        let (stderr, stderr_stream) = ScriptStderr::new();
        let wasi_ctx = WasiCtxBuilder::new()
            .stdout(stdout_stream)
            .stderr(stderr_stream)
            .build();

        let (doc, self_node_id, registry, events, agent_entry, doc_id, doc_entity) =
            if perms.api.contains(&ApiName::LocalAgent) {
                let Some(ref ad) = agent_docs else {
                    warn!(name, "local agent perms set but LocalAgentDocs not ready");
                    continue;
                };

                // Build bone proxy node IDs and registry without Loro.
                let mut bone_nodes = HashMap::new();
                let mut bone_node_ids = HashMap::new();
                let agent_registry = SceneRegistryInner::new();
                for &bone in ad.bone_entities.keys() {
                    let id = gen_id();
                    bone_nodes.insert(bone, id.clone());
                    bone_node_ids.insert(id.clone(), bone);
                    let inner = Arc::new(NodeInner {
                        id: id.clone(),
                        dirty: false.into(),
                        tree_id: Mutex::new(None),
                        state: Mutex::new(NodeState::default()),
                        entity: Mutex::new(None),
                    });
                    agent_registry
                        .nodes
                        .lock()
                        .expect("nodes lock")
                        .push(Arc::clone(&inner));
                    agent_registry
                        .node_map
                        .lock()
                        .expect("node_map lock")
                        .insert(id, Arc::clone(&inner));
                }

                let entry = Arc::new(AgentDocEntry {
                    bone_nodes: Arc::new(bone_nodes),
                    bone_node_ids: Arc::new(bone_node_ids),
                    registry: Arc::clone(&agent_registry),
                });
                ad.docs
                    .lock()
                    .expect("agent doc lock")
                    .push(Arc::clone(&entry));

                // Fresh root node for the script's "self".
                let self_node_id = gen_id();
                let self_inner = Arc::new(NodeInner {
                    id: self_node_id.clone(),
                    dirty: false.into(),
                    tree_id: Mutex::new(None),
                    state: Mutex::new(NodeState::default()),
                    entity: Mutex::new(None),
                });
                agent_registry
                    .nodes
                    .lock()
                    .expect("nodes lock")
                    .push(Arc::clone(&self_inner));
                agent_registry
                    .node_map
                    .lock()
                    .expect("node_map lock")
                    .insert(self_node_id.clone(), Arc::clone(&self_inner));

                // Dummy LoroDoc — no subscription; commit() is a no-op.
                let dummy_doc = Arc::new(LoroDoc::new());
                let doc_id = blake3::hash(&dummy_doc.peer_id().to_le_bytes());

                // Spawn doc entity without HsdDoc — init_hsd_doc won't touch it.
                let doc_ent = commands
                    .spawn((
                        HsdRecordId(doc_id),
                        SceneRegistry(Arc::clone(&agent_registry)),
                    ))
                    .id();

                // Build initial NodeAdded events for all bone proxy nodes + self.
                let mut init_events: Vec<DocChange> = agent_registry
                    .nodes
                    .lock()
                    .expect("nodes lock")
                    .iter()
                    .map(|inner| DocChange {
                        doc: doc_ent,
                        kind: DocChangeKind::NodeAdded {
                            id: inner.id.clone(),
                            parent_id: None,
                            data: HsdNodeData::default(),
                        },
                    })
                    .collect();

                let agent_events = Arc::new(Mutex::new(Vec::new()));
                let doc_event_queue = Arc::clone(&agent_events);

                // Drain init events into the queue.
                agent_events
                    .lock()
                    .expect("events lock")
                    .append(&mut init_events);

                commands
                    .entity(doc_ent)
                    .insert(DocChangeQueue(Arc::clone(&doc_event_queue)));

                (
                    dummy_doc,
                    self_node_id,
                    agent_registry,
                    agent_events,
                    Some(entry),
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
        );
        let state = StoreState::new(wasi_ctx, rt);

        let mut store = Store::new(&engine.0, state);
        store.epoch_deadline_async_yield_and_update(1);

        let component = wasmtime::component::Component::from_binary(&engine.0, &wasm.0);

        let rt = ScriptRuntime::new(store, stdout, stderr);
        let ctx = Arc::clone(&rt.ctx);
        commands.entity(ent).insert((LoadingScript, rt));

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
