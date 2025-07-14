use bevy::prelude::*;
use wasmtime::Config;

mod execute;
mod load;
mod wasm;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(wasm::WasmLoader)
            .init_asset::<wasm::Wasm>()
            .init_non_send_resource::<execute::Runtimes>()
            .add_systems(Startup, add_unavi_system)
            .add_systems(
                Update,
                (execute::execute_script_updates, load::load_scripts),
            );
    }
}

fn add_unavi_system(asset_server: Res<AssetServer>, mut commands: Commands) {
    let engine = {
        let mut config = Config::new();
        config.async_support(true).epoch_interruption(true);

        let Ok(engine) = wasmtime::Engine::new(&config) else {
            error!("Error creating wasmtime engine");
            return;
        };

        commands.spawn(WasmEngine(engine)).id()
    };

    let bin = asset_server.load("wasm/unavi/system.wasm");
    commands.spawn((Script(engine), WasmBinary(bin), Name::new("unavi:system")));
}

#[derive(Component)]
#[require(EngineScripts)]
pub struct WasmEngine(wasmtime::Engine);

#[derive(Component, Default)]
#[relationship_target(relationship = Script)]
pub struct EngineScripts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = EngineScripts)]
pub struct Script(pub Entity);

#[derive(Component)]
pub struct WasmBinary(pub Handle<wasm::Wasm>);
