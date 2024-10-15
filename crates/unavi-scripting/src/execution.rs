use bevy::{prelude::*, tasks::block_on};
use wasm_bridge::component::ResourceAny;

use crate::{data::ScriptControl, load::LoadedScript};

use super::load::ScriptMap;

pub fn init_scripts(
    mut commands: Commands,
    to_init: Query<
        (Entity, &Name),
        (
            With<LoadedScript>,
            Without<FailedToInit>,
            Without<ScriptResource>,
        ),
    >,
    script_map: NonSendMut<ScriptMap>,
) {
    for (entity, name) in to_init.iter() {
        let mut scripts = script_map.lock().unwrap();
        let script = scripts.get_mut(&entity).expect("Script not found");

        #[allow(clippy::await_holding_lock)]
        let result = block_on(async {
            info!("Initializing script {}", name);

            script
                .script
                .wired_script_types()
                .script()
                .call_constructor(&mut script.store)
                .await
        });

        let result = match result {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to construct script resource: {}", e);
                commands.entity(entity).insert(FailedToInit);
                continue;
            }
        };

        script.store.data().push_commands(&mut commands);

        commands.entity(entity).insert(ScriptResource(result));
    }
}

pub fn update_scripts(
    mut commands: Commands,
    mut scripts: Query<(Entity, &Name, &ScriptResource, &mut ScriptTickrate)>,
    script_map: NonSendMut<ScriptMap>,
) {
    for (entity, name, resource, mut tickrate) in scripts.iter_mut() {
        if !tickrate.ready_for_update {
            continue;
        }

        tickrate.ready_for_update = false;

        #[allow(clippy::await_holding_lock)]
        let result: anyhow::Result<_> = block_on(async {
            let span = trace_span!("ScriptUpdate", name = name.to_string());
            let span = span.enter();

            trace!("Updating script");

            let mut scripts = script_map.lock().unwrap();
            let script = scripts.get_mut(&entity).expect("Script not found");

            let result = script
                .script
                .wired_script_types()
                .script()
                .call_update(&mut script.store, resource.0, tickrate.delta)
                .await;

            script.store.data().push_commands(&mut commands);

            while let Ok(control) = script.store.data().control_recv.try_recv() {
                match control {
                    ScriptControl::Exit(e) => {
                        error!("Controlled exit: {}", e);
                        todo!("stop script execution");
                    }
                }
            }

            trace!("Done");
            drop(span);

            result
        });

        if let Err(e) = result {
            error!("Failed to update script: {}", e);
        };
    }
}

#[derive(Component)]
pub struct ScriptTickrate {
    delta: f32,
    last: f32,
    /// Ticks per second.
    tps: f32,
    pub ready_for_update: bool,
}

impl Default for ScriptTickrate {
    fn default() -> Self {
        Self {
            delta: 0.0,
            last: 0.0,
            tps: 1.0 / 20.0,
            ready_for_update: true,
        }
    }
}

pub fn tick_scripts(time: Res<Time>, mut scripts: Query<&mut ScriptTickrate>) {
    let now = time.elapsed_seconds();

    for mut tickrate in scripts.iter_mut() {
        if tickrate.last == 0.0 {
            tickrate.last = now;
            continue;
        }

        let delta = now - tickrate.last;

        if delta > tickrate.tps {
            tickrate.delta = delta;
            tickrate.last = now;
            tickrate.ready_for_update = true;
        };
    }
}

#[derive(Component)]
pub struct FailedToInit;

#[derive(Component)]
pub struct ScriptResource(ResourceAny);
