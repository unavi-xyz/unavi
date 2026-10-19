//! The lifecycle driver.
//!
//! A `jco` guest runs on the page's own thread, and a call suspended on a host
//! import resumes on the browser's microtask queue, which drains only after the
//! frame. A tick is polled inline so a host call it makes reaches this frame's
//! [`pump`]; one still suspended is kept and polled again next frame. That
//! leaves a script one call per frame, which
//! [`next_tick_one_call_per_frame`] spends on the update unless a fixed tick
//! has just come due.

use std::{
    future::Future,
    pin::Pin,
    time::Duration,
};

use bevy::{
    platform::{
        collections::HashMap,
        time::Instant,
    },
    prelude::*,
    tasks::futures_lite::future::{
        block_on,
        poll_once,
    },
};
use wasm_bindgen::JsValue;

use super::instantiate::{
    ScriptHostHandle,
    ScriptInstance,
};
use crate::{
    ScriptStatus,
    bindings::web::{
        HostHandle,
        describe_js_error,
        script_fixed_update,
        script_init,
        script_update,
        wit_tick,
    },
    engine::{
        Tick,
        TickClocks,
        TickKind,
        next_tick_one_call_per_frame,
    },
};

/// Time a frame spends applying the host calls its scripts made.
const PUMP_BUDGET: Duration = Duration::from_millis(2);

/// A lifecycle call suspended on a host import, kept until it finishes.
struct PendingCall(Pin<Box<dyn Future<Output = ()>>>);

// Safe: wasm32 has one thread, and this holds the script's `Rc` host and
// `JsValue` instance, as [`ScriptInstance`] does.
unsafe impl Send for PendingCall {}
unsafe impl Sync for PendingCall {}

/// Suspended calls, keyed by the script that owns them.
#[derive(Resource, Default)]
pub struct PendingCalls(HashMap<Entity, PendingCall>);

/// Polls each script's suspended call, starts the one it is due for, and
/// leaves whatever is still suspended for the next frame.
pub fn drive(
    time: Res<Time<Real>>,
    mut pending: ResMut<PendingCalls>,
    mut scripts: Query<(
        &ScriptStatus,
        &ScriptInstance,
        &ScriptHostHandle,
        &mut TickClocks,
        NameOrEntity,
    )>,
) {
    let now = time.elapsed();

    // A script despawned mid-call leaves its future behind.
    pending.0.retain(|entity, _| scripts.contains(*entity));

    for (status, instance, host, mut clocks, name) in &mut scripts {
        let entity = name.entity;

        // Poll a suspended call before starting a new one: one call at a time.
        if let Some(call) = pending.0.get_mut(&entity) {
            if block_on(poll_once(&mut call.0)).is_none() {
                continue;
            }
            pending.0.remove(&entity);
        }

        let Some(kind) = next_tick_one_call_per_frame(status, &mut clocks, now) else {
            continue;
        };
        if !status.enter() {
            continue;
        }
        let tick = clocks.next(kind, now);

        let mut call: Pin<Box<dyn Future<Output = ()>>> = Box::pin(run_lifecycle(
            kind,
            tick,
            status.clone(),
            instance.0.clone(),
            host.0.clone(),
            name.to_string(),
        ));
        if block_on(poll_once(&mut call)).is_none() {
            pending.0.insert(entity, PendingCall(call));
        }
    }
}

/// Applies the host calls scripts made this frame, so an `await` settles before
/// the next frame instead of waiting for `PreUpdate`.
pub fn pump(world: &mut World) {
    bevy_async::commands::pump(world, Instant::now() + PUMP_BUDGET);
}

async fn run_lifecycle(
    kind: TickKind,
    tick: Tick,
    status: ScriptStatus,
    js_instance: JsValue,
    host: HostHandle,
    name: String,
) {
    // Opened before the call and held across its `await`, exactly as native
    // holds its store lock for the call: a tick suspended between two host
    // calls must never show half its writes. The `RefCell` borrow itself does
    // not need to span the `await` — `open_tick` only needs `&mut ScriptHost`
    // to build the guard, which then owns its own `Arc` clones.
    let guard = host.borrow_mut().open_tick(tick.time);
    let result = match kind {
        TickKind::Init => script_init(&js_instance).await.map(drop),
        TickKind::Update => script_update(&js_instance, wit_tick(tick)).await.map(drop),
        TickKind::FixedUpdate => script_fixed_update(&js_instance, wit_tick(tick))
            .await
            .map(drop),
    };
    drop(guard);

    match (kind, result) {
        (TickKind::Init, Ok(())) => status.set_initialized(),
        // `init`'s only WIT error is a plain string the guest returned
        // (`runtime.ts` unwraps `ComponentError` for this reason); anything
        // else is a real trap.
        (TickKind::Init, Err(err)) if status.trap() => {
            if let Some(message) = err.as_string() {
                error!(
                    name = %name,
                    message,
                    "Script failed to initialize; it will not run"
                );
            } else {
                error!(
                    name = %name,
                    err = describe_js_error(&err),
                    export = kind.export(),
                    "Script trapped; it will not run again"
                );
            }
        }
        (_, Err(err)) if status.trap() => {
            error!(
                name = %name,
                err = describe_js_error(&err),
                export = kind.export(),
                "Script trapped; it will not run again"
            );
        }
        (_, Ok(()) | Err(_)) => {}
    }
    status.leave();
}
