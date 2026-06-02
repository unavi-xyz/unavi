use std::sync::Arc;

use bevy::prelude::*;
use tracing::Instrument;
use unavi_util::async_task::spawn_async_task;
use wasmtime::AsContextMut;

use crate::engine::{
    InitializedScript,
    native::instantiate::{
        ScriptGuest,
        ScriptSpan,
        ScriptStore,
    },
};

#[derive(Component)]
pub struct InitingScript(tokio::sync::oneshot::Receiver<()>);

pub fn init_scripts(
    to_init: Query<
        (Entity, &ScriptGuest, &ScriptStore, &ScriptSpan),
        (Without<InitingScript>, Without<InitializedScript>),
    >,
    mut commands: Commands,
) {
    for (entity, guest, store, span) in to_init {
        let guest = Arc::clone(&guest.0);
        let store = Arc::clone(&store.0);

        let (tx, rx) = tokio::sync::oneshot::channel();

        spawn_async_task(
            async move {
                let mut store = store.lock().await;
                store.set_epoch_deadline(1);

                match guest
                    .wired_script_guest_api()
                    .call_init(store.as_context_mut())
                    .await
                {
                    Ok(()) => {
                        drop(store);
                        let _ = tx.send(());
                    }
                    Err(err) => {
                        error!(?err, "Failed to init script");
                    }
                }
            }
            .instrument(span.0.clone()),
        );

        commands.entity(entity).insert(InitingScript(rx));
    }
}

pub fn poll_initing_scripts(scripts: Query<(Entity, &mut InitingScript)>, mut commands: Commands) {
    for (entity, mut initializing) in scripts {
        let Ok(()) = initializing.0.try_recv() else {
            continue;
        };

        commands
            .entity(entity)
            .remove::<InitingScript>()
            .insert(InitializedScript);
    }
}
