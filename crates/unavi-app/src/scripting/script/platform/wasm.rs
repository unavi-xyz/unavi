use bevy::prelude::*;

use crate::scripting::asset::Wasm;

use super::{logic, HostScripts, ScriptLoadQueue, ScriptsVec, WasmEngine};

pub fn load_scripts(
    engine: Res<WasmEngine>,
    host_scripts: Res<HostScripts>,
    load_queue: ResMut<ScriptLoadQueue>,
    scripts: NonSend<Scripts>,
    wasm_assets: Res<Assets<Wasm>>,
) {
    logic::load_scripts(
        engine,
        host_scripts,
        load_queue,
        scripts.0.clone(),
        wasm_assets,
    );
}

pub fn update_scripts(scripts: NonSend<Scripts>, last_time: Local<f32>, time: Res<Time>) {
    logic::update_scripts(scripts.0.clone(), last_time, time);
}
