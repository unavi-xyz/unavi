use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{HsdChild, HsdRecordId, NodeId, ScriptNode};
use tokio::sync::Mutex;
use tracing::{Instrument, Span};
use unavi_util::async_task::spawn_async_task;
use wasmtime::{Store, component::Linker};
use wasmtime_wasi::{ResourceTable, WasiCtxBuilder};

use crate::{
    Script,
    engine::{
        ScriptEngine,
        native::{
            WasmtimeEngine,
            log::{ScriptStderr, ScriptStdout},
            tick::LastTick,
        },
    },
    load::asset::Wasm,
    permissions::ApiPermissions,
    runtime::{
        Runtime,
        native::{NativeRuntime, add_apis_to_linker},
        shared::{
            RuntimeBackend,
            wired::scene::{SceneContext, WiredSceneBackend},
        },
    },
};

#[derive(Component, Deref, DerefMut)]
pub struct InstantiatingScript(tokio::sync::oneshot::Receiver<bindings::Guest>);

#[derive(Component, Deref, DerefMut)]
pub struct ScriptStore(pub Arc<Mutex<Store<Runtime>>>);

#[derive(Component)]
#[require(LastTick)]
pub struct ScriptGuest(pub Arc<bindings::Guest>);

#[derive(Component)]
pub struct ScriptSpan(pub Span);

pub fn instantiate_scripts(
    wasms: Res<Assets<Wasm>>,
    engines: Query<&WasmtimeEngine>,
    to_instantiate: Query<
        (Entity, &Script, &ScriptEngine, NameOrEntity, &ScriptNode),
        (Without<InstantiatingScript>, Without<ScriptGuest>),
    >,
    nodes: Query<(&NodeId, &HsdChild)>,
    docs: Query<(&HsdRecordId, Option<&ApiPermissions>)>,
    mut commands: Commands,
) {
    for (entity, script, engine_ent, name, node_ent) in to_instantiate {
        let Some(wasm) = wasms.get(&script.0) else {
            continue;
        };
        let Ok((node_id, node_doc)) = nodes.get(node_ent.0) else {
            continue;
        };
        let Ok((doc_id, perms)) = docs.get(node_doc.0) else {
            continue;
        };
        let Ok(engine) = engines.get(engine_ent.0) else {
            warn_once!("Can't instantiate: no engine");
            continue;
        };

        let span = info_span!("", name = name.to_string());

        let (stdout, stdout_stream) = ScriptStdout::new();
        let (stderr, stderr_stream) = ScriptStderr::new();
        stdout.drain(span.clone());
        stderr.drain(span.clone());

        let wasi_ctx = WasiCtxBuilder::new()
            .stdout(stdout_stream)
            .stderr(stderr_stream)
            .allow_tcp(false)
            .allow_udp(false)
            .build();

        let perms = perms.cloned().unwrap_or_default();

        let state = Runtime {
            backend: RuntimeBackend {
                wired_input: Arc::default(),
                wired_scene: Arc::new(Mutex::new(WiredSceneBackend::new(SceneContext {
                    perms: perms.clone(),
                    self_doc: doc_id.0,
                    self_node: node_id.0,
                }))),
            },
            native: NativeRuntime {
                table: ResourceTable::default(),
                wasi_ctx,
            },
        };
        let store = Arc::new(Mutex::new(Store::new(&engine.0, state)));

        let engine = engine.0.clone();
        let wasm = wasm.0.clone();

        let (tx, rx) = tokio::sync::oneshot::channel();

        spawn_async_task({
            let store = Arc::clone(&store);
            async move {
                let mut store = store.lock().await;
                match instantiate_component(&engine, &wasm, &mut store, &perms).await {
                    Ok(g) => {
                        let _ = tx.send(g);
                    }
                    Err(err) => error!(?err, "Failed to instantiate component"),
                }
            }
            .instrument(span.clone())
        });

        commands.entity(entity).insert((
            InstantiatingScript(rx),
            ScriptStore(store),
            ScriptSpan(span),
        ));
    }
}

mod bindings {
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

async fn instantiate_component(
    engine: &wasmtime::Engine,
    binary: &[u8],
    store: &mut Store<Runtime>,
    perms: &ApiPermissions,
) -> anyhow::Result<bindings::Guest> {
    let component = wasmtime::component::Component::from_binary(engine, binary)?;

    let mut linker = Linker::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    add_apis_to_linker(&mut linker, perms)?;

    info!("Instantiating script");
    let guest = bindings::Guest::instantiate_async(store, &component, &linker).await?;

    Ok(guest)
}

pub fn poll_instantiating(
    instantiating: Query<(Entity, &mut InstantiatingScript)>,
    mut commands: Commands,
) {
    for (entity, mut rx) in instantiating {
        let Ok(guest) = rx.try_recv() else {
            continue;
        };
        commands
            .entity(entity)
            .remove::<InstantiatingScript>()
            .insert(ScriptGuest(Arc::new(guest)));
    }
}
