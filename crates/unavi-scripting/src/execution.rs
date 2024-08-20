use bevy::{prelude::*, tasks::block_on};
use wasm_bridge::{component::ResourceAny, AsContextMut};

use crate::load::LoadedScript;

use super::load::Scripts;

#[derive(Component)]
pub struct FailedToInit;

#[derive(Component)]
pub struct ScriptResource(ResourceAny);

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
    scripts: NonSendMut<Scripts>,
) {
    for (entity, name) in to_init.iter() {
        #[allow(clippy::await_holding_lock)]
        let res = block_on(async {
            info!("Initializing script {}", name);

            let mut scripts = scripts.lock().unwrap();
            let (script, store) = scripts.get_mut(&entity).unwrap();

            script
                .wired_script_types()
                .script()
                .call_constructor(store)
                .await
        });

        if let Err(e) = res {
            error!("Failed to construct script resource: {}", e);
            commands.entity(entity).insert(FailedToInit);
            continue;
        };

        commands.entity(entity).insert(ScriptResource(res.unwrap()));
    }
}

pub fn update_scripts(
    mut commands: Commands,
    mut to_update: Query<(Entity, &Name, &ScriptResource)>,
    scripts: NonSendMut<Scripts>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();

    for (entity, name, res) in to_update.iter_mut() {
        #[allow(clippy::await_holding_lock)]
        let res: anyhow::Result<_> = block_on(async {
            let span = trace_span!("ScriptUpdate", name = name.to_string(), delta);
            let span = span.enter();

            trace!("Updating script");

            let mut scripts = scripts.lock().unwrap();
            let (script, store) = scripts.get_mut(&entity).unwrap();

            let res = script
                .wired_script_types()
                .script()
                .call_update(store.as_context_mut(), res.0, delta)
                .await;

            commands.append(&mut store.data_mut().commands);

            trace!("Done");
            drop(span);

            res
        });

        if let Err(e) = res {
            error!("Failed to update script: {}", e);
        };
    }
}
