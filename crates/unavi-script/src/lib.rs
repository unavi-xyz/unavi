use bevy::prelude::*;
use wasmtime::Config;

mod api;
mod asset;
mod event;
mod execute;
mod load;

pub use event::LoadScriptAsset;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        let mut config = Config::new();
        config.async_support(true).epoch_interruption(true);

        let engine = match wasmtime::Engine::new(&config) {
            Ok(e) => e,
            Err(e) => {
                error!("Error creating wasmtime engine: {e:?}");
                return;
            }
        };

        app.world_mut().spawn(WasmEngine(engine));

        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .add_event::<event::LoadScriptAsset>()
            .add_systems(
                FixedUpdate,
                (
                    event::handle_loads,
                    execute::increment_epochs,
                    execute::execute_script_updates,
                    load::load_scripts,
                ),
            );
    }
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
pub struct WasmBinary(pub Handle<asset::Wasm>);
