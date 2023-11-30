use bevy::prelude::*;

use super::{
    scripts::{InstantiatedScript, WasmStore},
    state::ScriptState,
};

pub fn init_scripts(
    mut scripts: Query<(&Parent, &mut InstantiatedScript)>,
    mut stores: Query<&mut WasmStore<ScriptState>>,
) {
    for (parent, mut script) in scripts.iter_mut() {
        if script.initialized {
            continue;
        }

        let mut store = match stores.get_mut(parent.get()) {
            Ok(store) => store,
            Err(e) => {
                error!("Failed to get store: {}", e);
                continue;
            }
        };

        script.initialized = true;

        if let Err(e) = script.script.call_init(&mut store.0) {
            error!("Error calling init: {}", e);
            continue;
        }
    }
}

pub fn update_scripts(
    time: Res<Time>,
    mut scripts: Query<(&Parent, &mut InstantiatedScript)>,
    mut stores: Query<&mut WasmStore<ScriptState>>,
) {
    for (parent, script) in scripts.iter_mut() {
        let mut store = match stores.get_mut(parent.get()) {
            Ok(store) => store,
            Err(e) => {
                error!("Failed to get store: {}", e);
                continue;
            }
        };

        if let Err(e) = script
            .script
            .call_update(&mut store.0, time.delta_seconds())
        {
            error!("Error calling update: {}", e);
            continue;
        }
    }
}

pub fn exit_scripts(
    mut commands: Commands,
    mut scripts: Query<(Entity, &Parent, &mut InstantiatedScript)>,
    mut stores: Query<&mut WasmStore<ScriptState>>,
) {
    for (entity, parent, script) in scripts.iter_mut() {
        let mut store = match stores.get_mut(parent.get()) {
            Ok(store) => store,
            Err(e) => {
                error!("Failed to get store: {}", e);
                continue;
            }
        };

        if !store.0.data().exit {
            continue;
        }

        if let Err(e) = script.script.call_cleanup(&mut store.0) {
            error!("Error calling cleanup: {}", e);
            continue;
        }

        commands.entity(entity).despawn();
    }
}
