use std::{sync::Arc, time::Duration};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use tracing::Instrument;
use wasmtime::{AsContextMut, error::Context};

use bevy_hsd::hydrate::events::ScriptCommandQueueComp;

use crate::{agent::NeedsAgentProxy, load::LoadedScript, runtime::ScriptRuntime};

#[derive(Component)]
pub struct InitializedScript;

#[derive(Component)]
pub struct InitializingScript {
    started: Duration,
    task: Task<anyhow::Result<()>>,
}

const MAX_INIT_DURATION: Duration = Duration::from_mins(1);

pub fn begin_init_scripts(
    mut commands: Commands,
    time: Res<Time>,
    to_init: Query<
        (Entity, &LoadedScript, &ScriptRuntime, Option<&Name>),
        (
            Without<InitializedScript>,
            Without<InitializingScript>,
            Without<NeedsAgentProxy>,
        ),
    >,
) {
    for (entity, loaded, rt, name) in to_init {
        let ctx = Arc::clone(&rt.ctx);
        let guest = Arc::clone(&loaded.0);
        let name = name.map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);

        let pool = AsyncComputeTaskPool::get();

        let span = info_span!("init", name);
        let task = pool.spawn(
            async move {
                let mut ctx = ctx.lock().await;
                ctx.store.set_epoch_deadline(1);

                let script = guest
                    .wired_script_guest_api()
                    .script()
                    .call_constructor(ctx.store.as_context_mut())
                    .await
                    .with_context(|| format!("construct script {name}"));

                ctx.flush_logs().await;
                ctx.store.data().rt.wired_scene.doc.commit();

                ctx.script = Some(script?);
                drop(ctx);

                Ok(())
            }
            .instrument(span),
        );

        commands.entity(entity).insert(InitializingScript {
            task,
            started: time.elapsed(),
        });
    }
}

pub fn end_init_scripts(
    mut commands: Commands,
    time: Res<Time>,
    mut initializing: Query<(Entity, &mut InitializingScript, &ScriptCommandQueueComp)>,
) {
    for (entity, mut init, cmd_queue) in &mut initializing {
        if time
            .elapsed()
            .checked_sub(init.started)
            .expect("time overflow")
            > MAX_INIT_DURATION
        {
            warn!("script init took too long");
            commands.entity(entity).remove::<InitializingScript>();
            continue;
        }

        match block_on(poll_once(&mut init.task)) {
            Some(Ok(())) => {
                let mut q = cmd_queue.0.lock().expect("cmd queue lock");
                commands.append(&mut q.inner);
                q.len = 0;
                drop(q);
                commands
                    .entity(entity)
                    .remove::<InitializingScript>()
                    .insert(InitializedScript);
            }
            Some(Err(err)) => {
                error!(?err, "script init failed");
                commands.entity(entity).remove::<InitializingScript>();
            }
            None => {}
        }
    }
}
