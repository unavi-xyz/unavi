use bevy::prelude::*;

use crate::{EngineScripts, Script, ScriptStore, WasmEngine, load::LoadedScript};

pub fn tick_engine(
    engines: Query<(&WasmEngine, &EngineScripts)>,
    scripts: Query<(&LoadedScript, &ScriptStore)>,
) {
    for (engine, engine_scripts) in engines {
        for e in engine_scripts.0.iter() {
            let Ok((script, store)) = scripts.get(*e) else {
                continue;
            };
        }
    }
}
