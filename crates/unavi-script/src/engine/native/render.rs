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
    RenderTicking,
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

type RenderQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static RenderTicking,
        &'static ScriptGuest,
        &'static ScriptStore,
        &'static ScriptSpan,
    ),
    With<InitializedScript>,
>;

pub fn render_tick_scripts(
    world: &mut World,
    state: &mut SystemState<(Res<'static, Time<Real>>, RenderQuery<'static, 'static>)>,
) {
    let outstanding = Arc::new(AtomicUsize::new(0));

    let budget = {
        let Ok((time, to_tick)) = state.get_mut(world) else {
            return;
        };
        let budget = script_budget(&time);

        for (ticking, guest, store, span) in to_tick {
            if ticking.0.swap(true, Ordering::SeqCst) {
                continue;
            }

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
                        .call_render(store.as_context_mut())
                        .await
                    {
                        warn!(?err, "Failed to render tick script");
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
