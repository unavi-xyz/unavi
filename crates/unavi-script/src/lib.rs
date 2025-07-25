use bevy::prelude::*;
use wasmtime::Config;

mod api;
mod asset;
mod commands;
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
                    load::load_scripts,
                    (
                        execute::increment_epochs,
                        execute::execute_script_updates,
                        commands::process_commands,
                    )
                        .chain(),
                ),
            );
    }
}

#[derive(Component)]
#[require(Scripts)]
pub struct WasmEngine(wasmtime::Engine);

#[derive(Component, Default)]
#[relationship_target(relationship = ScriptEngine)]
pub struct Scripts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Scripts)]
pub struct ScriptEngine(pub Entity);

#[derive(Component)]
pub struct WasmBinary(pub Handle<asset::Wasm>);
