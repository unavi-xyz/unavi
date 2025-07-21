use bevy::prelude::*;

use crate::{Script, WasmBinary, WasmEngine};

/// Load a script from the assets folder.
#[derive(Event)]
pub struct LoadScriptAsset {
    pub namespace: &'static str,
    pub package: &'static str,
}

pub fn handle_loads(
    mut events: EventReader<LoadScriptAsset>,
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
            Script(engine),
            WasmBinary(bin),
            Name::new(format!("{namespace}:{package}")),
        ));
    }
}
