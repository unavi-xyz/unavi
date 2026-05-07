use std::sync::{Arc, atomic::Ordering};

use bevy::prelude::*;
use tracing::Instrument;
use unavi_util::async_task::spawn_async_task;
use wasmtime::AsContextMut;

use crate::{
    RenderTicking,
    engine::native::{
        construct::ScriptResource,
        instantiate::{ScriptGuest, ScriptSpan, ScriptStore},
    },
};

pub fn render_tick_scripts(
    to_tick: Query<(
        &RenderTicking,
        &ScriptGuest,
        &ScriptStore,
        &ScriptResource,
        &ScriptSpan,
    )>,
) {
    for (ticking, guest, store, res, span) in to_tick {
        if ticking.0.swap(true, Ordering::Relaxed) {
            continue;
        }

        let ticking = Arc::clone(&ticking.0);
        let guest = Arc::clone(&guest.0);
        let res = res.0;
        let store = Arc::clone(&store.0);

        spawn_async_task(
            async move {
                let mut store = store.lock().await;
                store.set_epoch_deadline(1);

                if let Err(err) = guest
                    .wired_script_guest_api()
                    .script()
                    .call_render(store.as_context_mut(), res)
                    .await
                {
                    warn!(?err, "Failed to render tick script");
                }

                store.data().api.doc.commit();
                drop(store);

                ticking.store(false, Ordering::Relaxed);
            }
            .instrument(span.0.clone()),
        );
    }
}
