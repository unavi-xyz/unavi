use bevy::prelude::*;
use bevy::transform::TransformSystems;
use wasmtime::Config;

pub mod agent;
pub mod api;
pub mod runtime;

#[derive(Component)]
#[require(crate::Scripts)]
pub struct WasmEngine(pub wasmtime::Engine);

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

        app.world_mut().spawn(WasmEngine(engine));

        app.add_observer(crate::load::native::init::on_hsd_record_removed)
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
            .add_observer(agent::on_avatar_bones_added)
            .add_systems(PreUpdate, runtime::increment_epochs)
            .add_systems(Update, runtime::render::render_tick_scripts)
            .add_systems(
                PostUpdate,
                agent::reset_bone_proxies.before(TransformSystems::Propagate),
            )
            // FixedUpdate chain — order matters:
            //   firewall sync → local/HSD loading → proxy init → init → tick → events
            // Must run after init_hsd_doc so scene state is hydrated before scripts tick.
            .add_systems(
                FixedUpdate,
                (
                    crate::firewall::sync_hsd_firewall_entities,
                    (
                        crate::load::local::poll_local_scripts,
                        crate::load::native::hsd::load_hsd_scripts,
                        crate::load::native::hsd::cleanup_hsd_scripts,
                        crate::load::native::init::register_new_docs,
                        crate::load::native::init::load_scripts,
                        agent::init_agent_proxies,
                        runtime::init::begin_init_scripts,
                        runtime::init::end_init_scripts,
                        runtime::tick::tick_scripts,
                        crate::event_registry::process_event_emissions,
                    )
                        .chain()
                        .after(bevy_hsd::hydrate::init::init_hsd_doc),
                ),
            );
    }
}
