use std::{sync::Arc, time::Duration};

use bevy::{ecs::world::CommandQueue, prelude::*};
use tracing::Instrument;
use unavi_util::{async_commands::ASYNC_COMMAND_QUEUE, async_task::spawn_async_task};
use wasmtime::AsContextMut;

use crate::engine::native::{
    Executed, Executing,
    init::ScriptResource,
    instantiate::{ScriptGuest, ScriptSpan, ScriptStore},
};

const TICKRATE: Duration = Duration::from_millis(50);

#[derive(Component, Default)]
pub struct LastTick(f32);

pub fn tick_scripts(
    time: Res<Time>,
    to_tick: Query<
        (
            Entity,
            &ScriptGuest,
            &ScriptStore,
            &ScriptResource,
            &ScriptSpan,
            &mut LastTick,
        ),
        Without<Executing>,
    >,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();

    for (entity, guest, store, res, span, mut last) in to_tick {
        if now - last.0 < TICKRATE.as_secs_f32() {
            continue;
        }

        last.0 = now;

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
                    .call_tick(store.as_context_mut(), res)
                    .await
                {
                    warn!(?err, "Failed to tick script");
                }
                drop(store);

                let mut queue = CommandQueue::default();
                queue.push(bevy::ecs::system::command::trigger(Executed(entity)));
                let _ = ASYNC_COMMAND_QUEUE.0.send(queue).await;
            }
            .instrument(span.0.clone()),
        );

        commands.entity(entity).insert(Executing);
    }
}
