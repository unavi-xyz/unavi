//! Builds each script's store and instance.

use std::sync::Arc;

use bevy::prelude::*;
use bevy_async::{
    AsyncWorld,
    task,
};
use bevy_hsd::{
    document::{
        Hsd,
        HsdDocId,
    },
    prim::{
        Prim,
        PrimOf,
    },
};
use smol_str::SmolStr;
use tokio::sync::{
    Mutex,
    oneshot,
};
use tracing::{
    Instrument,
    Span,
};
use unavi_policy::quota::Quota;
use unavi_space::{
    authority::SpaceView,
    identity::RootDocument,
};
use wasmtime::{
    Store,
    UpdateDeadline,
};
use wasmtime_wasi::WasiCtxBuilder;

use crate::{
    Script,
    ScriptStatus,
    bindings::native::{
        HostCtx,
        Shell,
    },
    engine::native::{
        WasmtimeEngine,
        log::{
            ScriptStderr,
            ScriptStdout,
        },
    },
    host::{
        ScriptHost,
        ScriptIdentity,
        SharedResources,
    },
    load::asset::Wasm,
    quota::{
        QuotaExempt,
        limiter::QuotaLimiter,
    },
};

/// Frames one call may run for before the guest is stopped. Waiting on a host
/// call does not count, only running guest code.
const MAX_CALL_EPOCHS: u32 = 60;

/// A script's store, shared with the task running its current call.
#[derive(Component, Clone)]
pub struct ScriptStore(pub Arc<Mutex<Store<HostCtx>>>);

/// A script's instance, once instantiated.
#[derive(Component, Clone)]
pub struct ScriptInstance(pub Arc<Shell>);

#[derive(Component)]
pub struct ScriptSpan(pub Span);

#[derive(Component)]
pub struct Instantiating(oneshot::Receiver<Shell>);

pub fn instantiate_scripts(
    engine: Res<WasmtimeEngine>,
    wasms: Res<Assets<Wasm>>,
    scripts: Query<
        (Entity, &Script, &ScriptStatus, NameOrEntity, &Prim, &PrimOf),
        (Without<Instantiating>, Without<ScriptStore>),
    >,
    docs: Query<(&HsdDocId, &Hsd, Has<QuotaExempt>)>,
    root: Option<Res<RootDocument>>,
    view: Option<Res<SpaceView>>,
    shared: SharedResources,
    async_world: Res<AsyncWorld>,
    mut commands: Commands,
) {
    let Some(view) = view else {
        return;
    };

    for (entity, script, status, name, prim, prim_of) in scripts {
        if status.is_trapped() {
            continue;
        }
        let Some(wasm) = wasms.get(&script.0) else {
            continue;
        };
        let Ok((doc_id, doc, exempt)) = docs.get(prim_of.0) else {
            continue;
        };

        let name = name.to_string();
        let span = info_span!("", name);
        let (stdout, stdout_stream) = ScriptStdout::new();
        let (stderr, stderr_stream) = ScriptStderr::new();
        stdout.drain(SmolStr::new(&name));
        stderr.drain(SmolStr::new(&name));
        let wasi_ctx = WasiCtxBuilder::new()
            .stdout(stdout_stream)
            .stderr(stderr_stream)
            .allow_tcp(false)
            .allow_udp(false)
            .build();

        let quota = if exempt {
            Quota::unlimited()
        } else {
            view.document_quota(doc_id.0)
        };
        let host = ScriptHost::new(
            ScriptIdentity {
                doc:      doc_id.0,
                prim:     prim.0,
                state:    Arc::clone(&doc.0),
                view:     (*view).clone(),
                quota:    Arc::clone(&quota),
                world:    (*async_world).clone(),
                root_doc: root.as_ref().map(|root| root.0),
            },
            shared.get(),
        );

        let mut store = Store::new(
            &engine.engine,
            HostCtx::new(host, QuotaLimiter::new(quota), wasi_ctx),
        );
        store.limiter(|ctx| &mut ctx.limiter);
        // A guest yields to the executor once a frame, and is stopped once a
        // single call has run for too many of them.
        store.epoch_deadline_callback(|mut ctx| {
            let ctx = ctx.data_mut();
            ctx.epochs += 1;
            Ok(if ctx.epochs > MAX_CALL_EPOCHS {
                UpdateDeadline::Interrupt
            } else {
                UpdateDeadline::Yield(1)
            })
        });
        store.set_epoch_deadline(1);
        let store = Arc::new(Mutex::new(store));

        let (tx, rx) = oneshot::channel();
        let engine = engine.clone();
        let (bytes, hash) = (Arc::clone(&wasm.bytes), wasm.hash);
        let task_store = Arc::clone(&store);
        let status = status.clone();
        task::spawn(
            async move {
                let mut store = task_store.lock().await;
                let instance = async {
                    let pre = engine.prepare(hash, &bytes)?;
                    pre.instantiate_async(&mut *store).await
                };
                match instance.await {
                    Ok(instance) => {
                        let _ = tx.send(instance);
                    }
                    Err(err) => {
                        if status.trap() {
                            error!(?err, "Failed to instantiate the script; it will not run");
                        }
                    }
                }
            }
            .instrument(span.clone()),
        );

        commands
            .entity(entity)
            .insert((Instantiating(rx), ScriptStore(store), ScriptSpan(span)));
    }
}

pub fn finish_instantiating(
    mut scripts: Query<(Entity, &mut Instantiating)>,
    mut commands: Commands,
) {
    for (entity, mut instantiating) in &mut scripts {
        match instantiating.0.try_recv() {
            Ok(instance) => {
                commands
                    .entity(entity)
                    .remove::<Instantiating>()
                    .insert(ScriptInstance(Arc::new(instance)));
            }
            Err(oneshot::error::TryRecvError::Empty) => {}
            Err(oneshot::error::TryRecvError::Closed) => {
                commands.entity(entity).remove::<Instantiating>();
            }
        }
    }
}
