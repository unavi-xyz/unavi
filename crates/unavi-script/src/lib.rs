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
            .add_observer(commands::cleanup_vobjects)
            .add_systems(PreUpdate, execute::increment_epochs)
            .add_systems(
                FixedUpdate,
                (
                    event::handle_load_events,
                    execute::init::begin_init_scripts,
                    execute::init::end_init_scripts,
                    load::load_scripts,
                ),
            )
            .add_systems(FixedPostUpdate, commands::process_commands)
            .add_systems(PostUpdate, commands::process_commands);
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
