use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use bevy::prelude::*;
use tracing::Instrument;
use unavi_util::async_task::spawn_async_task;
use wasmtime::AsContextMut;

use crate::{
    Ticking,
    engine::native::{
        init::InitializedScript,
        instantiate::{ScriptGuest, ScriptSpan, ScriptStore},
    },
};

const TICKRATE: Duration = Duration::from_millis(50);

#[derive(Component, Default)]
pub struct LastTick(Duration);

pub fn tick_scripts(
    time: Res<Time>,
    to_tick: Query<
        (
            &Ticking,
            &ScriptGuest,
            &ScriptStore,
            &ScriptSpan,
            &mut LastTick,
        ),
        With<InitializedScript>,
    >,
) {
    let now = time.elapsed();

    for (ticking, guest, store, span, mut last) in to_tick {
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

                store.data().api.doc.commit();
                drop(store);

                ticking.store(false, Ordering::SeqCst);
            }
            .instrument(span.0.clone()),
        );
    }
}
