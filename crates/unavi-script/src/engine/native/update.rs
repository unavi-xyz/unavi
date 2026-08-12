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
use tracing::Instrument;
use unavi_util::async_task::spawn_async_task;
use wasmtime::AsContextMut;

use crate::{
    Updating,
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

type UpdateQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Updating,
        &'static ScriptGuest,
        &'static ScriptStore,
        &'static ScriptSpan,
    ),
    With<InitializedScript>,
>;

pub fn update_scripts(
    world: &mut World,
    state: &mut SystemState<(Res<'static, Time<Real>>, UpdateQuery<'static, 'static>)>,
) {
    let outstanding = Arc::new(AtomicUsize::new(0));

    let budget = {
        let Ok((time, to_update)) = state.get_mut(world) else {
            return;
        };
        let budget = script_budget(&time);

        for (updating, guest, store, span) in to_update {
            if updating.0.swap(true, Ordering::SeqCst) {
                continue;
            }

            let updating = Arc::clone(&updating.0);
            let guest = Arc::clone(&guest.0);
            let store = Arc::clone(&store.0);
            let outstanding = Arc::clone(&outstanding);
            outstanding.fetch_add(1, Ordering::AcqRel);

            spawn_async_task(
                async move {
                    let mut store = store.lock().await;
                    store.set_epoch_deadline(1);

                    let api = Arc::clone(&store.data().api);
                    let tick = api.open_tick().await;
                    let result = guest
                        .wired_script_guest_api()
                        .call_update(store.as_context_mut())
                        .await;
                    drop(tick);

                    if let Err(err) = result {
                        warn!(?err, "Failed to update script");
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
