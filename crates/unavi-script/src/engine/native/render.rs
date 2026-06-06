use std::sync::{
    Arc,
    atomic::Ordering,
};

use bevy::prelude::*;
use tracing::Instrument;
use unavi_util::async_task::spawn_async_task;
use wasmtime::AsContextMut;

use crate::{
    RenderTicking,
    engine::{
        InitializedScript,
        native::instantiate::{
            ScriptGuest,
            ScriptSpan,
            ScriptStore,
        },
    },
};

pub fn render_tick_scripts(
    to_tick: Query<
        (&RenderTicking, &ScriptGuest, &ScriptStore, &ScriptSpan),
        With<InitializedScript>,
    >,
) {
    for (ticking, guest, store, span) in to_tick {
        if ticking.0.swap(true, Ordering::SeqCst) {
            continue;
        }

        let ticking = Arc::clone(&ticking.0);
        let guest = Arc::clone(&guest.0);
        let store = Arc::clone(&store.0);

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
            }
            .instrument(span.0.clone()),
        );
    }
}
