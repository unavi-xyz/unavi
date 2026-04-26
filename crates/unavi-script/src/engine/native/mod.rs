use bevy::prelude::*;
use wasmtime::Config;

use crate::engine::Engine;

mod log;

pub struct NativeEnginePlugin;

impl Plugin for NativeEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_wasmtime_engine)
            .add_systems(PreUpdate, increment_epochs);
    }
}

#[derive(Component)]
struct WasmtimeEngine(wasmtime::Engine);

fn init_wasmtime_engine(trigger: On<Add, Engine>, mut commands: Commands) {
    let engine = match wasmtime::Engine::new(Config::default().epoch_interruption(true)) {
        Ok(e) => e,
        Err(err) => {
            error!(?err, "Failed to create Wasmtime engine");
            return;
        }
    };

    commands
        .entity(trigger.entity)
        .insert(WasmtimeEngine(engine));
}

fn increment_epochs(engines: Query<&WasmtimeEngine>) {
    for engine in engines {
        engine.0.increment_epoch();
    }
}
