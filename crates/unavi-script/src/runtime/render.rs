use std::sync::Arc;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use futures::FutureExt;
use wasmtime::AsContextMut;

use crate::{
    load::LoadedScript,
    runtime::{ScriptRuntime, init::InitializedScript, tick::TickingTask},
};

pub fn render_tick_scripts(
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

        let ctx = Arc::clone(&rt.ctx);
        let name = name.map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);
        let pool = AsyncComputeTaskPool::get();

        let task = pool.spawn(async move {
            let mut ctx = ctx.lock().await;
            ctx.store.set_epoch_deadline(1);

            let script = ctx.script.expect("initialized script resource");

            let span = info_span!("render", name);
            let span = span.enter();

            guest
                .wired_script_guest_api()
                .script()
                .call_render(ctx.store.as_context_mut(), script)
                .await?;

            ctx.flush_logs().await;
            ctx.store.data().rt.wired_scene.doc.commit();
            ctx.store.data().rt.wired_scene.doc.compact_change_store();
            ctx.store.data().rt.wired_scene.doc.free_history_cache();
            drop(ctx);
            drop(span);

            Ok(())
        });

        if let Some(prev) = ticking.0.take()
            && let Some(res) = prev.now_or_never()
            && let Err(err) = res
        {
            error!(?err, "error render ticking script");
        }

        ticking.0 = Some(task);
    }
}
