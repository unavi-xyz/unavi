//! The lifecycle driver, sharing [`crate::engine::next_tick`] and
//! [`crate::engine::lane_for`] with `engine::native` so both engines agree on
//! when a script is called.

use bevy::prelude::*;
use bevy_async::task;

use super::instantiate::{
    ScriptHostHandle,
    ScriptInstance,
};
use crate::{
    ScriptStatus,
    bindings::web::{
        describe_js_error,
        script_fixed_update,
        script_init,
        script_update,
        wit_tick,
    },
    engine::{
        Schedule,
        TickClocks,
        TickKind,
        lane_for,
        next_tick,
    },
};

/// The schedules [`drive`] runs in, matching `engine::native::tick`.
pub const UPDATE: u8 = 0;
pub const FIXED: u8 = 1;

/// Starts each due script's call for this schedule. Unlike native, nothing
/// here blocks the frame for a call to finish: a `jco`-transpiled guest runs
/// on this same JS thread regardless, so there is no separate executor to
/// wait on, only the microtask queue the browser already drains between
/// frames.
pub fn drive<const SCHEDULE: u8>(
    time: Res<Time<Real>>,
    mut scripts: Query<(
        &ScriptStatus,
        &ScriptInstance,
        &ScriptHostHandle,
        &mut TickClocks,
        NameOrEntity,
    )>,
) {
    let now = time.elapsed();
    let schedule = if SCHEDULE == UPDATE {
        Schedule::Update
    } else {
        Schedule::Fixed
    };

    for (status, instance, host, mut clocks, name) in &mut scripts {
        let Some(kind) = next_tick(schedule, status, &clocks, now) else {
            continue;
        };
        let lane = lane_for(kind);
        if !status.begin(lane) {
            continue;
        }
        let tick = clocks.next(kind, now);

        let status = status.clone();
        let js_instance = instance.0.clone();
        let host = std::rc::Rc::clone(&host.0);
        let name = name.to_string();
        task::spawn(async move {
            // Opened before the call and held across its `await`, exactly as
            // native holds its store lock for the call: a tick suspended
            // between two host calls must never show half its writes. The
            // `RefCell` borrow itself does not need to span the `await` —
            // `open_tick` only needs `&mut ScriptHost` to build the guard,
            // which then owns its own `Arc` clones.
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
                // `init`'s only WIT error is a plain string the guest
                // returned (`runtime.ts` unwraps `ComponentError` for this
                // reason); anything else is a real trap.
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
            status.end(lane);
        });
    }
}
