//! The UNAVI desktop and web client: windowing, identity, the scene state
//! machine, and the glue that assembles every other crate into one app.

// Bevy's `AsBindGroup` needs a higher limit than the derive's default.
#![recursion_limit = "256"]

use std::sync::Arc;

use bevy::{
    light::light_consts::lux,
    log::LogPlugin,
    prelude::*,
    window::WindowTheme,
};
use bevy_iroh::endpoint::LoadEndpoint;
use iroh::endpoint_info::AddrFilter;
use tracing::Level;

mod camera;
mod config;
mod fade;
mod icon;
mod identity;
mod scene;

#[cfg(feature = "devtools")] mod devtools;

#[cfg(not(target_family = "wasm"))] mod xr;

pub struct UnaviPlugin {
    pub in_memory: bool,
    pub join:      Option<String>,
    pub log_level: Level,
    pub xr:        bool,
}

const DISABLED_LOGS: &[&str] = &[
    "cranelift_codegen",
    "offset_allocator",
    "wasmtime_internal_cranelift",
];

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        let mut filter = DISABLED_LOGS
            .iter()
            .map(|s| format!("{s}=off"))
            .collect::<Vec<_>>();
        filter.push(bevy::log::DEFAULT_FILTER.to_string());

        let default_plugins = DefaultPlugins
            .set(LogPlugin {
                filter: filter.join(","),
                level: self.log_level,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    name: Some("unavi".to_string()),
                    title: "UNAVI".to_string(),
                    window_theme: Some(WindowTheme::Dark),
                    // On the web this is every browser shortcut at once,
                    // reload and dev tools included. `unavi_input` holds back
                    // the few keys the app actually binds instead.
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            });

        // Registers the `iroh://` asset source, which must exist before
        // `AssetPlugin` builds the sources it knows about.
        app.add_plugins(unavi_assets::AssetsPlugin);

        cfg_select! {
            target_family = "wasm" => {
                let default_plugins = default_plugins.set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                });
                app.add_plugins(default_plugins);
            }
            _ => {
                let default_plugins =
                    default_plugins.disable::<bevy::asset::io::web::WebAssetPlugin>();
                if self.xr {
                    app.add_plugins((
                        bevy_mod_openxr::add_xr_plugins(default_plugins),
                        xr::XrPlugin,
                    ));
                } else {
                    app.add_plugins(default_plugins);
                }
            }
        }

        #[cfg(feature = "devtools")]
        app.add_plugins(devtools::ClientDevToolsPlugin);

        // Built once, shared with every plugin that persists opaque app
        // state: the identity keys and the trust table live here.
        let storage = identity::key_storage(self.in_memory);
        // Input config is hand-edited, so it gets the config directory
        // instead of sharing `storage`'s data directory.
        let config_storage = identity::config_storage();

        app.add_plugins((
            unavi_physics::PhysicsPlugin,
            bevy_hsd::HsdPlugin,
            bevy_iroh::IrohPlugin,
            unavi_agent::AgentPlugin,
            unavi_avatar::AvatarPlugin,
            identity::IdentityPlugin {
                storage: storage.clone(),
                sync:    config::sync_config(),
            },
            unavi_input::InputPlugin {
                storage: Some(config_storage),
            },
            unavi_portal::PortalPlugin,
            unavi_script::ScriptPlugin,
            unavi_space::SpacePlugin {
                storage: Some(storage),
            },
            bevy_async::AsyncPlugin,
        ))
        .add_plugins((
            camera::CameraPlugin,
            fade::FadePlugin,
            unavi_grab::GrabPlugin,
            scene::ScenePlugin,
        ))
        .insert_resource(scene::home::JoinSpace(self.join.clone()))
        .insert_resource(GlobalAmbientLight {
            brightness: lux::OVERCAST_DAY,
            ..default()
        })
        .configure_sets(
            Update,
            unavi_script::ScriptSystems::Snapshot.after(unavi_agent::AgentMovementSet),
        )
        .add_systems(Startup, icon::set_window_icon);

        load_endpoint(app);
    }
}

/// The endpoint key derives from the identity, so the identity plugin has to
/// have built before this runs. `IdentityPlugin` skips inserting these
/// resources when it could not build a DID resolver; degrade to a clean exit
/// instead of panicking on the missing resource.
fn load_endpoint(app: &mut App) {
    let Some(secret_key) = app
        .world()
        .get_resource::<identity::LocalNode>()
        .map(|node| node.0.endpoint().clone())
    else {
        error!("identity did not initialize; exiting");
        app.world_mut().write_message(AppExit::error());
        return;
    };
    let Some(auth) = app
        .world()
        .get_resource::<identity::Auth>()
        .map(|auth| Arc::clone(&auth.0))
    else {
        error!("identity did not initialize; exiting");
        app.world_mut().write_message(AppExit::error());
        return;
    };

    app.world_mut().trigger(LoadEndpoint {
        configure: Some(Arc::new(move |builder| auth.install(builder))),
        filter: AddrFilter::default(),
        secret_key,
    });
}
