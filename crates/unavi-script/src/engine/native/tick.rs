use std::{
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
    time::Duration,
};

use bevy::{
    ecs::system::SystemState,
    prelude::*,
};
use tracing::Instrument;
use unavi_util::async_task::spawn_async_task;
use wasmtime::AsContextMut;

use crate::{
    Ticking,
    engine::{
        InitializedScript,
        native::{
            drive::{
                script_budget,
                wait_for_scripts,
            },
            instantiate::{
                ScriptGuest,
                ScriptSpan,
                ScriptStore,
            },
        },
    },
};

const TICKRATE: Duration = Duration::from_millis(50);

#[derive(Component, Default)]
pub struct LastTick(Duration);

type TickQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Ticking,
        &'static ScriptGuest,
        &'static ScriptStore,
        &'static ScriptSpan,
        &'static mut LastTick,
    ),
    With<InitializedScript>,
>;

pub fn tick_scripts(
    world: &mut World,
    state: &mut SystemState<(Res<'static, Time<Real>>, TickQuery<'static, 'static>)>,
) {
    let outstanding = Arc::new(AtomicUsize::new(0));

    let budget = {
        let Ok((time, mut to_tick)) = state.get_mut(world) else {
            return;
        };
        let budget = script_budget(&time);
        let now = time.elapsed();

        for (ticking, guest, store, span, mut last) in &mut to_tick {
            let delta = now.checked_sub(last.0).unwrap_or_default();
            if delta < TICKRATE {
                continue;
            }
            if ticking.0.swap(true, Ordering::SeqCst) {
                continue;
            }

            let margin = delta
                .checked_sub(TICKRATE)
                .expect("always greater")
                .min(TICKRATE);
            last.0 = now.checked_sub(margin).unwrap_or_default();

            let ticking = Arc::clone(&ticking.0);
            let guest = Arc::clone(&guest.0);
            let store = Arc::clone(&store.0);
            let outstanding = Arc::clone(&outstanding);
            outstanding.fetch_add(1, Ordering::AcqRel);

            spawn_async_task(
                async move {
                    let mut store = store.lock().await;
                    store.set_epoch_deadline(1);

                    if let Err(err) = guest
                        .wired_script_guest_api()
                        .call_tick(store.as_context_mut())
                        .await
                    {
                        warn!(?err, "Failed to tick script");
                    }
                    drop(store);

                    ticking.store(false, Ordering::SeqCst);
                    outstanding.fetch_sub(1, Ordering::AcqRel);
                }
                .instrument(span.0.clone()),
            );
        }

        budget
    };

    wait_for_scripts(world, &outstanding, budget);
}
