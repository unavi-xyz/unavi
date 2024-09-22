use bevy::{prelude::*, tasks::block_on};
use wasm_bridge::{component::ResourceAny, AsContextMut};

use crate::load::LoadedScript;

use super::load::ScriptMap;

pub(crate) fn init_scripts(
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
        #[allow(clippy::await_holding_lock)]
        let result = block_on(async {
            info!("Initializing script {}", name);

            let mut scripts = script_map.lock().unwrap();
            let (script, store) = scripts.get_mut(&entity).expect("Script not found");

            script
                .wired_script_types()
                .script()
                .call_constructor(store)
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

        commands.entity(entity).insert(ScriptResource(result));
    }
}

pub(crate) fn update_scripts(
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
            let span = trace_span!("ScriptUpdate", name = name.to_string(), tickrate.delta);
            let span = span.enter();

            trace!("Updating script");

            let mut scripts = script_map.lock().unwrap();
            let (script, store) = scripts.get_mut(&entity).expect("Script not found");

            let result = script
                .wired_script_types()
                .script()
                .call_update(store.as_context_mut(), resource.0, tickrate.delta)
                .await;

            commands.append(&mut store.data_mut().commands);

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
    pub ready_for_update: bool,
    tickrate: f32,
}

impl Default for ScriptTickrate {
    fn default() -> Self {
        Self {
            delta: 0.0,
            last: 0.0,
            ready_for_update: true,
            tickrate: 1.0 / 20.0,
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

        if delta > tickrate.tickrate {
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
