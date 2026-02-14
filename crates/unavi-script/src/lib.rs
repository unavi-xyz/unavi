use bevy::prelude::*;
use wasmtime::Config;

mod api;
mod asset;
mod event;
mod load;
mod runtime;

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
            .add_message::<event::LoadScriptAsset>()
            .add_systems(
                PreUpdate,
                (
                    // commands::apply_wasm_commands,
                    runtime::increment_epochs,
                    // (
                    //     commands::system::tick_script_cycle,
                    //     commands::system::start_script_cycle,
                    // )
                    //     .chain(),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    event::handle_load_events,
                    load::load_scripts,
                    runtime::init::begin_init_scripts,
                    runtime::init::end_init_scripts,
                    runtime::tick::tick_scripts,
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
