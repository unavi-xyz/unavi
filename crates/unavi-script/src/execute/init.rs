use std::time::Duration;

use anyhow::Context;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use wasmtime::AsContextMut;

use crate::{commands::system::ScriptRuntime, load::LoadedScript};

#[derive(Component)]
pub struct InitializedScript;

#[derive(Component)]
pub struct InitializingScript {
    started: Duration,
    task: Task<anyhow::Result<()>>,
}

const MAX_INIT_DURATION: Duration = Duration::from_secs(60);

pub fn begin_init_scripts(
    mut commands: Commands,
    time: Res<Time>,
    to_init: Query<
        (Entity, &LoadedScript, &ScriptRuntime, Option<&Name>),
        (Without<InitializedScript>, Added<LoadedScript>),
    >,
) {
    for (entity, loaded, rt, name) in to_init {
        let ctx = rt.ctx.clone();
        let guest = loaded.0.clone();
        let name = name.map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);

        let pool = AsyncComputeTaskPool::get();

        let task = pool.spawn(async move {
            let mut ctx = ctx.lock().await;
            ctx.store.set_epoch_deadline(1);

            info!("Constructing script {name}");

            let script = guest
                .wired_ecs_guest_api()
                .script()
                .call_constructor(ctx.store.as_context_mut())
                .await
                .with_context(|| format!("construct script {name}"));

            ctx.flush_logs().await;

            ctx.script = Some(script?);
            drop(ctx);

            Ok(())
        });

        commands.entity(entity).insert(InitializingScript {
            task,
            started: time.elapsed(),
        });
    }
}

pub fn end_init_scripts(
    mut commands: Commands,
    time: Res<Time>,
    mut initializing: Query<(Entity, &mut InitializingScript)>,
) {
    for (entity, mut init) in &mut initializing {
        if time
            .elapsed()
            .checked_sub(init.started)
            .expect("time overflow")
            > MAX_INIT_DURATION
        {
            warn!("Script init took too long");
            commands.entity(entity).remove::<InitializingScript>();
            continue;
        }

        match block_on(poll_once(&mut init.task)) {
            Some(Ok(())) => {
                commands
                    .entity(entity)
                    .remove::<InitializingScript>()
                    .insert(InitializedScript);
            }
            Some(Err(e)) => {
                error!("Script init error: {e:?}");
                commands.entity(entity).remove::<InitializingScript>();
            }
            None => {}
        }
    }
}
