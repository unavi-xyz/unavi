use std::{sync::Arc, time::Duration};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures::FutureExt;
use tracing::Instrument;
use wasmtime::AsContextMut;

use crate::{
    load::LoadedScript,
    runtime::{ScriptRuntime, init::InitializedScript},
};

const SCRIPT_TICKRATE: Duration = Duration::from_millis(50);

#[derive(Component, Default)]
pub struct TickingTask(pub Option<Task<anyhow::Result<()>>>);

pub fn tick_scripts(
    time: Res<Time>,
    scripts: Query<
        (
            &ScriptRuntime,
            &LoadedScript,
            &mut TickingTask,
            Option<&Name>,
        ),
        With<InitializedScript>,
    >,
) {
    for (rt, loaded, mut ticking, name) in scripts {
        if let Some(task) = &ticking.0
            && !task.is_finished()
        {
            continue;
        }

        let guest = Arc::clone(&loaded.0);

        {
            let mut ctx = rt.ctx.blocking_lock();

            if ctx.last_tick.is_zero() {
                ctx.last_tick = time.elapsed();
                continue;
            }

            let delta = time
                .elapsed()
                .checked_sub(ctx.last_tick)
                .expect("valid delta");
            if delta < SCRIPT_TICKRATE {
                continue;
            }

            drop(ctx);
        }

        let ctx = Arc::clone(&rt.ctx);
        let name = name.map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);
        let pool = AsyncComputeTaskPool::get();

        let span = info_span!("tick", name);
        let task = pool.spawn(
            async move {
                let mut ctx = ctx.lock().await;
                ctx.store.set_epoch_deadline(1);

                let script = ctx.script.expect("initialized script resource");

                guest
                    .wired_script_guest_api()
                    .script()
                    .call_tick(ctx.store.as_context_mut(), script)
                    .await?;

                ctx.flush_logs().await;
                drop(ctx);

                Ok(())
            }
            .instrument(span),
        );

        if let Some(prev) = ticking.0.take()
            && let Some(res) = prev.now_or_never()
            && let Err(err) = res
        {
            error!(?err, "error ticking script");
        }

        ticking.0 = Some(task);
    }
}
