use bevy::prelude::*;

use crate::{ScriptEngine, WasmBinary, WasmEngine};

/// Load a script from the assets folder.
#[derive(Message)]
pub struct LoadScriptAsset {
    pub namespace: &'static str,
    pub package: &'static str,
}

pub fn handle_load_events(
    mut events: MessageReader<LoadScriptAsset>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    engine: Query<Entity, With<WasmEngine>>,
) {
    for LoadScriptAsset { namespace, package } in events.read() {
        let engine = engine
            .single()
            .expect("wasm engine should be spawned on startup");
        let bin = asset_server.load(format!("wasm/{namespace}/{package}.wasm"));
        commands.spawn((
            ScriptEngine(engine),
            WasmBinary(bin),
            Name::new(format!("{namespace}:{package}")),
        ));
    }
}
