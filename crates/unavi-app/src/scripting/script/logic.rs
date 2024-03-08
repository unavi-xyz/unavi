use std::sync::{Arc, Mutex};

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use wasm_bridge::AsContextMut;

use crate::scripting::asset::Wasm;

use super::{ScriptLoadQueue, ScriptsVec, WasmEngine, WasmScript};

pub fn load_scripts(
    scripts: ScriptsVec,
    engine: Res<WasmEngine>,
    mut load_queue: ResMut<ScriptLoadQueue>,
    wasm_assets: Res<Assets<Wasm>>,
) {
    if load_queue.0.is_empty() {
        return;
    }

    let mut to_remove = Vec::new();

    let pool = AsyncComputeTaskPool::get();

    for handle in &load_queue.0 {
        if let Some(wasm) = wasm_assets.get(handle) {
            to_remove.push(handle.clone());

            let bytes = wasm.bytes.clone();
            let engine = engine.0.clone();
            let scripts = scripts.clone();

            pool.spawn(async move {
                let script = match WasmScript::new(&engine, &bytes).await {
                    Ok(script) => script,
                    Err(e) => {
                        error!("Failed to create script: {}", e);
                        return;
                    }
                };

                scripts.lock().unwrap().push(Arc::new(Mutex::new(script)));
            })
            .detach();
        }
    }

    for handle in to_remove {
        load_queue.0.retain(|h| h != &handle);
    }
}

const SCRIPT_HZ: f32 = 20.0;
const SCRIPT_DELTA: f32 = 1.0 / SCRIPT_HZ;

pub fn update_scripts(scripts: ScriptsVec, mut last_time: Local<f32>, time: Res<Time>) {
    let current_time = time.elapsed_seconds();
    let delta = current_time - *last_time;

    if delta < SCRIPT_DELTA {
        return;
    }

    *last_time = current_time;

    let pool = AsyncComputeTaskPool::get();

    for script in scripts.lock().unwrap().iter_mut() {
        let script = script.clone();

        #[allow(clippy::await_holding_lock)]
        pool.spawn_local(async move {
            let wasm_script = script.lock().unwrap();

            if !wasm_script.initialized {
                let mut store = wasm_script.store.lock().unwrap();

                if let Err(e) = wasm_script
                    .script
                    .interface0
                    .call_init(store.as_context_mut())
                    .await
                {
                    error!("Failed to initialize script: {}", e);
                    return;
                }

                let mut wasm_script = script.lock().unwrap();
                wasm_script.initialized = true;

                return;
            }

            let mut store = wasm_script.store.lock().unwrap();

            if let Err(e) = wasm_script
                .script
                .interface0
                .call_update(store.as_context_mut(), delta)
                .await
            {
                error!("Failed to update script: {}", e);
            }
        })
        .detach();
    }
}
