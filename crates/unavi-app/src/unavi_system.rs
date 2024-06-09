use bevy::prelude::*;
use unavi_scripting::{asset::Wasm, ScriptBundle};

const UNAVI_SYSTEM: &str = "unavi_system";

pub fn spawn_unavi_system(asset_server: Res<AssetServer>, mut commands: Commands) {
    let wasm = load_component_wasm(&asset_server, UNAVI_SYSTEM);

    commands.spawn((
        ScriptBundle {
            name: UNAVI_SYSTEM.into(),
            wasm,
        },
        SpatialBundle::default(),
    ));
}

pub fn load_component_wasm(asset_server: &AssetServer, name: &str) -> Handle<Wasm> {
    let path = format!("components/{}_{}.wasm", name, env!("CARGO_PKG_VERSION"));
    asset_server.load(path)
}
