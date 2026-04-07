use bevy::prelude::*;
use bevy::transform::TransformSystems;
use wasmtime::Config;

pub struct NativeScriptPlugin;

impl Plugin for NativeScriptPlugin {
    fn build(&self, app: &mut App) {
        let mut config = Config::new();
        config.epoch_interruption(true);

        let engine = match wasmtime::Engine::new(&config) {
            Ok(e) => e,
            Err(e) => {
                error!("Error creating wasmtime engine: {e:?}");
                return;
            }
        };

        app.world_mut().spawn(crate::WasmEngine(engine));

        app.add_observer(crate::load::on_hsd_record_removed)
            .init_resource::<crate::api::wired::scene::GlobalRegistryMapRes>()
            .add_observer(crate::api::wired::input::bridge::bridge_squeeze_down)
            .add_observer(crate::api::wired::input::bridge::bridge_squeeze_up)
            .add_systems(
                Update,
                (
                    crate::api::wired::input::bridge::update_menu_buffer,
                    crate::api::wired::input::bridge::update_squeeze_buffer,
                ),
            )
            .add_observer(crate::agent::on_avatar_bones_added)
            .add_systems(PreUpdate, crate::runtime::increment_epochs)
            .add_systems(Update, crate::runtime::render::render_tick_scripts)
            .add_systems(
                PostUpdate,
                crate::agent::reset_bone_proxies.before(TransformSystems::Propagate),
            )
            .add_systems(
                FixedUpdate,
                (
                    crate::firewall::sync_hsd_firewall_entities,
                    (
                        crate::load::local::poll_local_scripts,
                        crate::load::hsd::load_hsd_scripts,
                        crate::load::hsd::cleanup_hsd_scripts,
                        crate::load::register_new_docs,
                        crate::load::load_scripts,
                        crate::agent::init_agent_proxies,
                        crate::runtime::init::begin_init_scripts,
                        crate::runtime::init::end_init_scripts,
                        crate::runtime::tick::tick_scripts,
                        crate::api::wired::event::bridge::process_event_emissions,
                    )
                        .chain()
                        .after(bevy_hsd::hydrate::init::init_hsd_doc),
                ),
            );
    }
}
