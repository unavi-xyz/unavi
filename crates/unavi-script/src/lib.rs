use bevy::prelude::*;

mod engine;
mod load;
mod wasm;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(wasm::WasmLoader)
            .init_asset::<wasm::Wasm>()
            .add_systems(Startup, add_unavi_system)
            .add_systems(Update, (engine::tick_engine, load::load_scripts));
    }
}

fn add_unavi_system(asset_server: Res<AssetServer>, mut commands: Commands) {
    let engine = commands.spawn(WasmEngine::default()).id();

    let bin = asset_server.load("wasm/unavi/system.wasm");
    commands.spawn((Script(engine), WasmBinary(bin)));
}

#[derive(Component, Default)]
#[require(EngineScripts)]
pub struct WasmEngine(wasmtime::Engine);

#[derive(Component, Default)]
#[relationship_target(relationship = Script)]
pub struct EngineScripts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = EngineScripts)]
pub struct Script(pub Entity);

#[derive(Component)]
pub struct ScriptStore(pub wasmtime::Store<()>);

#[derive(Component)]
pub struct WasmBinary(pub Handle<wasm::Wasm>);
