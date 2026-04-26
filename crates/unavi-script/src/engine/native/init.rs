use std::sync::Arc;

use bevy::prelude::*;
use tracing::Instrument;
use unavi_util::async_task::spawn_async_task;
use wasmtime::{AsContextMut, component::ResourceAny};

use crate::engine::native::instantiate::{ScriptGuest, ScriptSpan, ScriptStore};

#[derive(Component)]
pub struct ScriptResource(pub ResourceAny);

#[derive(Component)]
pub struct InitializingScript(tokio::sync::oneshot::Receiver<ResourceAny>);

pub fn init_scripts(
    to_init: Query<
        (Entity, &ScriptGuest, &ScriptStore, &ScriptSpan),
        Or<(Without<InitializingScript>, Without<ScriptResource>)>,
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
                    .script()
                    .call_constructor(store.as_context_mut())
                    .await
                {
                    Ok(res) => {
                        let _ = tx.send(res);
                    }
                    Err(err) => {
                        error!(?err, "Failed to construct script resource");
                    }
                }
            }
            .instrument(span.0.clone()),
        );

        commands.entity(entity).insert(InitializingScript(rx));
    }
}

pub fn poll_initializing_scripts(
    scripts: Query<(Entity, &mut InitializingScript)>,
    mut commands: Commands,
) {
    for (entity, mut initializing) in scripts {
        let Ok(res) = initializing.0.try_recv() else {
            continue;
        };

        commands
            .entity(entity)
            .remove::<InitializingScript>()
            .insert(ScriptResource(res));
    }
}
