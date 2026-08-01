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
    FixedUpdating,
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

const FIXED_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Component, Default)]
pub struct LastFixedUpdate(Duration);

type FixedUpdateQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static FixedUpdating,
        &'static ScriptGuest,
        &'static ScriptStore,
        &'static ScriptSpan,
        &'static mut LastFixedUpdate,
    ),
    With<InitializedScript>,
>;

pub fn fixed_update_scripts(
    world: &mut World,
    state: &mut SystemState<(Res<'static, Time<Real>>, FixedUpdateQuery<'static, 'static>)>,
) {
    let outstanding = Arc::new(AtomicUsize::new(0));

    let budget = {
        let Ok((time, mut to_update)) = state.get_mut(world) else {
            return;
        };
        let budget = script_budget(&time);
        let now = time.elapsed();

        for (updating, guest, store, span, mut last) in &mut to_update {
            let delta = now.checked_sub(last.0).unwrap_or_default();
            if delta < FIXED_UPDATE_INTERVAL {
                continue;
            }
            if updating.0.swap(true, Ordering::SeqCst) {
                continue;
            }

            let margin = delta
                .checked_sub(FIXED_UPDATE_INTERVAL)
                .expect("always greater")
                .min(FIXED_UPDATE_INTERVAL);
            last.0 = now.checked_sub(margin).unwrap_or_default();

            let updating = Arc::clone(&updating.0);
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
                        .call_fixed_update(store.as_context_mut())
                        .await
                    {
                        warn!(?err, "Failed to fixed-update script");
                    }
                    drop(store);

                    updating.store(false, Ordering::SeqCst);
                    outstanding.fetch_sub(1, Ordering::AcqRel);
                }
                .instrument(span.0.clone()),
            );
        }

        budget
    };

    wait_for_scripts(world, &outstanding, budget);
}
