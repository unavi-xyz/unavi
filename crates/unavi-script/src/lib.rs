use bevy::{prelude::*, transform::TransformSystems};
use wasmtime::Config;

pub mod agent;
mod api;
mod asset;
pub mod firewall;
pub mod load;
pub mod permissions;
mod runtime;

pub use api::wired::event::EventRegistry;
pub use api::wired::input::{InputAction, InputDevice, InputRegistry, QueuedEvent};
pub use api::wired::scene::GlobalRegistryMapRes;
pub use load::local::{LoadLocalScript, ScriptSource};
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

        app.init_resource::<api::wired::event::EventRegistry>()
            .init_resource::<api::wired::input::InputRegistry>()
            .init_resource::<api::wired::scene::GlobalRegistryMapRes>()
            .add_observer(api::wired::input::bridge::bridge_squeeze_down)
            .add_observer(api::wired::input::bridge::bridge_squeeze_up)
            .add_systems(
                Update,
                (
                    api::wired::input::bridge::update_menu_buffer,
                    api::wired::input::bridge::update_squeeze_buffer,
                ),
            )
            .register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .add_observer(agent::on_avatar_bones_added)
            .add_observer(load::local::on_load_local_script)
            .add_observer(load::on_hsd_record_removed)
            .add_systems(PreUpdate, runtime::increment_epochs)
            .add_systems(Update, runtime::render::render_tick_scripts)
            .add_systems(
                PostUpdate,
                agent::reset_bone_proxies.before(TransformSystems::Propagate),
            )
            .add_systems(
                FixedUpdate,
                (
                    firewall::sync_hsd_firewall_entities,
                    (
                        load::local::poll_local_scripts,
                        load::hsd::load_hsd_scripts,
                        load::hsd::cleanup_hsd_scripts,
                        load::register_new_docs,
                        load::load_scripts,
                        agent::init_agent_proxies,
                        runtime::init::begin_init_scripts,
                        runtime::init::end_init_scripts,
                        runtime::tick::tick_scripts,
                        api::wired::event::bridge::process_event_emissions,
                    )
                        .chain()
                        .after(bevy_hsd::hydrate::init::init_hsd_doc),
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
