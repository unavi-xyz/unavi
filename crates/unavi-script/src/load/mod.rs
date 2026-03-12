use std::{collections::HashMap, sync::Arc, sync::Mutex, task::Poll};

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use bevy_hsd::{
    HsdDoc, HsdRecordId,
    cache::{NodeInner, NodeState, SceneRegistry, SceneRegistryInner},
    data::HsdNodeData,
    hydrate::events::{DocChange, DocChangeKind, DocEventQueue},
};
use log::{ScriptStderr, ScriptStdout};
use loro::{LoroTree, TreeParentId};
use state::{RuntimeData, StoreState};
use wasmtime::{AsContextMut, Store, component::Linker};
use wasmtime_wasi::WasiCtxBuilder;

use bevy_wds::{LocalActor, LocalBlobs};

use crate::{
    ScriptEngine, WasmBinary, WasmEngine,
    agent::{AgentDocEntry, AgentHsdDoc, LocalAgentDocs},
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
    doc_event_queues: Query<&DocEventQueue>,
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

                // Create a fresh LoroDoc for this script with proxy nodes for all bones.
                let new_doc = Arc::new(loro::LoroDoc::new());
                let mut bone_nodes = HashMap::new();
                {
                    let tree = new_doc
                        .get_map("hsd")
                        .get_or_create_container("nodes", LoroTree::new())
                        .expect("agent nodes tree");
                    for &bone in ad.bone_entities.keys() {
                        let node_id = tree.create(TreeParentId::Root).expect("bone node");
                        let meta = tree.get_meta(node_id).expect("meta");
                        let bone_str = format!("{bone}");
                        meta.insert("bone_name", bone_str.trim_matches('"'))
                            .expect("bone_name");
                        bone_nodes.insert(bone, node_id);
                    }
                    new_doc.commit();
                }

                // Pre-create SceneRegistry with NodeInners for each bone.
                let agent_registry = SceneRegistryInner::new();
                for &tree_id in bone_nodes.values() {
                    let inner = Arc::new(NodeInner {
                        dirty: false.into(),
                        tree_id,
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
                        .insert(tree_id, Arc::clone(&inner));
                }

                let entry = Arc::new(AgentDocEntry {
                    doc: Arc::clone(&new_doc),
                    bone_nodes: Arc::new(bone_nodes),
                    registry: Arc::clone(&agent_registry),
                });
                ad.docs
                    .lock()
                    .expect("agent doc lock")
                    .push(Arc::clone(&entry));

                // Fresh root node for the script's "self" in the agent doc.
                let self_node_id = {
                    let tree = new_doc
                        .get_map("hsd")
                        .get_or_create_container("nodes", LoroTree::new())
                        .expect("nodes");
                    tree.create(TreeParentId::Root).expect("script root node")
                };
                new_doc.commit();

                // Create NodeInner for self_node.
                let self_inner = Arc::new(NodeInner {
                    dirty: false.into(),
                    tree_id: self_node_id,
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
                    .insert(self_node_id, Arc::clone(&self_inner));

                let doc_id = blake3::hash(&new_doc.peer_id().to_le_bytes());

                // Spawn HsdDoc entity first to get the entity ID, then build NodeAdded
                // events with the known doc entity.
                let doc_ent = commands
                    .spawn((
                        AgentHsdDoc,
                        HsdDoc(Arc::clone(&new_doc)),
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
                    .map(|inner| {
                        let tree_id: smol_str::SmolStr =
                            format!("{}@{}", inner.tree_id.counter, inner.tree_id.peer).into();
                        DocChange {
                            doc: doc_ent,
                            kind: DocChangeKind::NodeAdded {
                                tree_id,
                                parent_id: None,
                                data: HsdNodeData::default(),
                            },
                        }
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
                    .insert(DocEventQueue(Arc::clone(&doc_event_queue)));

                (
                    new_doc,
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
                let Ok(event_queue) = doc_event_queues.get(source.doc_entity) else {
                    warn!("DocEventQueue not found for script");
                    continue;
                };
                let Ok(self_node_id) = loro::TreeID::try_from(source.tree_id.as_str()) else {
                    warn!("invalid tree id: {}", source.tree_id);
                    continue;
                };
                let doc = hsd_docs
                    .get(source.doc_entity)
                    .map_or_else(|_| Arc::new(loro::LoroDoc::new()), |hsd| Arc::clone(&hsd.0));

                let doc_id = hsd_record_ids
                    .get(source.doc_entity)
                    .map_or_else(|_| blake3::hash(b"unknown"), |r| r.0);

                (
                    doc,
                    self_node_id,
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
