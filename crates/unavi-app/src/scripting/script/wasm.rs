use bevy::prelude::*;

use crate::scripting::asset::Wasm;

use super::{logic, ScriptLoadQueue, ScriptsVec, WasmEngine};

#[derive(Default)]
pub struct Scripts(pub ScriptsVec);

pub fn load_scripts(
    engine: Res<WasmEngine>,
    load_queue: ResMut<ScriptLoadQueue>,
    scripts: NonSend<Scripts>,
    wasm_assets: Res<Assets<Wasm>>,
) {
    logic::load_scripts(scripts.0.clone(), engine, load_queue, wasm_assets);
}

pub fn update_scripts(scripts: NonSend<Scripts>, last_time: Local<f32>, time: Res<Time>) {
    logic::update_scripts(scripts.0.clone(), last_time, time);
}
