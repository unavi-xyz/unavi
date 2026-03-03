use bevy::prelude::*;
use wasmtime::Config;

mod api;
mod asset;
pub mod dev;
mod load;
pub mod permissions;
mod runtime;

pub use load::local::SpawnLocalScript;
pub use permissions::ScriptPermissions;

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

        app.init_resource::<load::local::PendingHandles>()
            .register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .add_observer(load::local::on_spawn_local_script)
            .add_systems(PreUpdate, runtime::increment_epochs)
            .add_systems(
                FixedUpdate,
                (
                    load::local::poll_local_scripts,
                    load::hsd::load_hsd_scripts,
                    load::hsd::cleanup_hsd_scripts,
                    load::load_scripts,
                    runtime::init::begin_init_scripts,
                    runtime::init::end_init_scripts,
                    runtime::tick::tick_scripts,
                )
                    .chain()
                    .after(bevy_hsd::init_hsd_doc),
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
