use std::sync::Arc;

use bevy::prelude::*;
use tracing::Instrument;
use unavi_util::{async_commands::AsyncCommands, async_task::spawn_async_task};
use wasmtime::AsContextMut;

use crate::engine::native::{
    Executed, Executing,
    init::ScriptResource,
    instantiate::{ScriptGuest, ScriptSpan, ScriptStore},
};

pub fn render_tick_scripts(
    to_tick: Query<
        (
            Entity,
            &ScriptGuest,
            &ScriptStore,
            &ScriptResource,
            &ScriptSpan,
        ),
        Without<Executing>,
    >,
    mut commands: Commands,
) {
    for (entity, guest, store, res, span) in to_tick {
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
                    warn!(?err, "Failed to tick script");
                }
                drop(store);

                let _ = AsyncCommands::default()
                    .trigger(Executed(entity))
                    .send()
                    .await;
            }
            .instrument(span.0.clone()),
        );

        commands.entity(entity).insert(Executing);
    }
}
