use bevy::prelude::*;
use wasm_bridge::component::ResourceAny;

use crate::load::LoadedScript;

use super::load::Scripts;

#[derive(Component)]
pub struct FailedToInit;

#[derive(Component)]
pub struct ScriptResource(ResourceAny);

pub fn init_scripts(
    mut commands: Commands,
    mut to_init: Query<
        Entity,
        (
            With<LoadedScript>,
            Without<FailedToInit>,
            Without<ScriptResource>,
        ),
    >,
    mut scripts: NonSendMut<Scripts>,
) {
    for entity in to_init.iter_mut() {
        let (script, store) = scripts.0.get_mut(&entity).unwrap();

        let res = match script.wired_script_types().script().call_constructor(store) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to construct script resource: {}", e);
                commands.entity(entity).insert(FailedToInit);
                continue;
            }
        };

        commands.entity(entity).insert(ScriptResource(res));
    }
}

const UPDATE_HZ: f32 = 60.0;
const UPDATE_DELTA: f32 = 1.0 / UPDATE_HZ;

pub fn update_scripts(
    mut last_update: Local<f32>,
    mut scripts: NonSendMut<Scripts>,
    mut to_update: Query<(Entity, &ScriptResource)>,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    let delta = now - *last_update;

    if delta < UPDATE_DELTA {
        return;
    }

    *last_update = now;

    for (entity, res) in to_update.iter_mut() {
        let (script, store) = scripts.0.get_mut(&entity).unwrap();

        if let Err(e) = script
            .wired_script_types()
            .script()
            .call_update(store, res.0, delta)
        {
            error!("Failed to update script: {}", e);
        };
    }
}
