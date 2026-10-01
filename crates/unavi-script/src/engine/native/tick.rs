//! The one driver for every lifecycle call.

use std::sync::{
    Arc,
    atomic::{
        AtomicUsize,
        Ordering,
    },
};

use bevy::{
    ecs::system::SystemState,
    prelude::*,
};
use bevy_async::task;
use tracing::Instrument;

use crate::{
    ScriptStatus,
    bindings::native::generated::exports::wired::script::lifecycle::Tick as WitTick,
    engine::{
        Tick,
        TickClocks,
        TickKind,
        native::{
            drive::{
                script_budget,
                wait_for_scripts,
            },
            instantiate::{
                ScriptInstance,
                ScriptSpan,
                ScriptStore,
            },
        },
    },
    status::Lane,
};

/// The schedules [`drive`] runs in.
pub const UPDATE: u8 = 0;
pub const FIXED: u8 = 1;

type Scripts<'w, 's> = Query<
    'w,
    's,
    (
        &'static ScriptStatus,
        &'static ScriptInstance,
        &'static ScriptStore,
        &'static ScriptSpan,
        &'static mut TickClocks,
    ),
>;

/// Starts each script's call for this schedule, then waits out the frame's
/// budget for them while pumping the host calls they make.
///
/// The fixed schedule runs `init` for a script that has not had it, and
/// `fixed-update` at [`crate::engine::FIXED_INTERVAL`]; the update schedule
/// runs `update` once a frame. A call still running when its next turn comes
/// skips that turn.
pub fn drive<const SCHEDULE: u8>(
    world: &mut World,
    state: &mut SystemState<(Res<'static, Time<Real>>, Scripts<'static, 'static>)>,
) {
    let outstanding = Arc::new(AtomicUsize::new(0));

    let budget = {
        let Ok((time, mut scripts)) = state.get_mut(world) else {
            return;
        };
        let now = time.elapsed();

        for (status, instance, store, span, mut clocks) in &mut scripts {
            if status.is_trapped() {
                continue;
            }
            let kind = match (SCHEDULE, status.is_initialized()) {
                (UPDATE, true) => TickKind::Update,
                (UPDATE, false) => continue,
                (_, false) => TickKind::Init,
                (_, true) if clocks.fixed_due(now) => TickKind::FixedUpdate,
                (_, true) => continue,
            };
            let lane = if kind == TickKind::Update {
                Lane::Update
            } else {
                Lane::Fixed
            };
            if !status.begin(lane) {
                continue;
            }
            let tick = clocks.next(kind, now);

            outstanding.fetch_add(1, Ordering::AcqRel);
            task::spawn(
                call(
                    kind,
                    tick,
                    lane,
                    status.clone(),
                    instance.clone(),
                    store.clone(),
                    Arc::clone(&outstanding),
                )
                .instrument(span.0.clone()),
            );
        }

        script_budget(&time)
    };

    wait_for_scripts(world, &outstanding, budget);
}

async fn call(
    kind: TickKind,
    tick: Tick,
    lane: Lane,
    status: ScriptStatus,
    instance: ScriptInstance,
    store: ScriptStore,
    outstanding: Arc<AtomicUsize>,
) {
    let mut store = store.0.lock().await;
    store.data_mut().epochs = 0;
    store.set_epoch_deadline(1);

    let guard = store.data_mut().host.open_tick(tick.time);
    let lifecycle = instance.0.wired_script_lifecycle();
    let wit_tick = WitTick {
        dt:    tick.dt,
        time:  tick.time,
        index: tick.index,
    };
    let result = match kind {
        TickKind::Init => lifecycle.call_init(&mut *store).await.map(|init| {
            init.map_or_else(
                |message| {
                    if status.trap() {
                        error!(message, "Script failed to initialize; it will not run");
                    }
                },
                |()| status.set_initialized(),
            );
        }),
        TickKind::Update => lifecycle.call_update(&mut *store, wit_tick).await,
        TickKind::FixedUpdate => lifecycle.call_fixed_update(&mut *store, wit_tick).await,
    };
    drop(guard);
    drop(store);

    if let Err(err) = result
        && status.trap()
    {
        error!(
            ?err,
            export = kind.export(),
            "Script trapped; it will not run again"
        );
    }
    status.end(lane);
    outstanding.fetch_sub(1, Ordering::AcqRel);
}
