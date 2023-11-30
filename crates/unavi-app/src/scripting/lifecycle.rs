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
