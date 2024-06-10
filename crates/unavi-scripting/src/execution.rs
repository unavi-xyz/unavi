use bevy::prelude::*;
use wasm_component_layer::{AsContextMut, ResourceOwn, Value};

use super::{load::WasmStores, script::ScriptInterface};

#[derive(Component)]
pub struct ScriptData(ResourceOwn);

#[derive(Component)]
pub struct FailedToInit;

pub fn init_scripts(
    mut commands: Commands,
    mut to_init: Query<(Entity, &ScriptInterface), (Without<ScriptData>, Without<FailedToInit>)>,
    mut stores: NonSendMut<WasmStores>,
) {
    for (entity, script) in to_init.iter_mut() {
        let store = stores.0.get_mut(&entity).unwrap();

        let mut results = vec![Value::U8(0)];

        if let Err(e) = script.init.call(store.as_context_mut(), &[], &mut results) {
            error!("Failed to init script: {}", e);
            commands.entity(entity).insert(FailedToInit);
            continue;
        }

        let script_data = match results.remove(0) {
            Value::Own(own) => own,
            _ => {
                error!("Wrong script data value");
                commands.entity(entity).insert(FailedToInit);
                continue;
            }
        };

        commands.entity(entity).insert(ScriptData(script_data));
    }
}

const UPDATE_HZ: f32 = 60.0;
const UPDATE_DELTA: f32 = 1.0 / UPDATE_HZ;

pub fn update_scripts(
    mut last_update: Local<f32>,
    mut scripts: Query<(Entity, &ScriptInterface, &ScriptData)>,
    mut stores: NonSendMut<WasmStores>,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    let delta = now - *last_update;

    if delta < UPDATE_DELTA {
        return;
    }

    *last_update = now;

    for (entity, script, data) in scripts.iter_mut() {
        let store = stores.0.get_mut(&entity).unwrap();

        let script_data_borrow = match data.0.borrow(store.as_context_mut()) {
            Ok(s) => Value::Borrow(s),
            Err(e) => {
                error!("Failed to borrow script data: {}", e);
                continue;
            }
        };

        if let Err(e) = script.update.call(
            store.as_context_mut(),
            &[Value::F32(delta), script_data_borrow],
            &mut [],
        ) {
            error!("Failed to call script update: {}", e);
        }
    }
}
