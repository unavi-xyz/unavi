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
mod fade;
mod icon;
mod scene;
mod secrets;

#[cfg(feature = "devtools")] mod dev_tools;

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
        app.add_plugins(unavi_assets_fetch::UnaviAssetsPlugin);

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
        app.add_plugins(dev_tools::ClientDevToolsPlugin);

        app.add_plugins((
            unavi_physics::PhysicsPlugin,
            bevy_hsd::HsdPlugin,
            bevy_iroh::IrohPlugin,
            bevy_wds::WdsPlugin,
            unavi_agent::AgentPlugin,
            unavi_avatar::AvatarPlugin,
            unavi_identity::IdentityPlugin {
                in_memory: self.in_memory,
                sync:      secrets::sync_config(),
            },
            unavi_input::InputPlugin,
            unavi_manifold::ManifoldPlugin,
            unavi_script::ScriptPlugin,
            unavi_space::SpacePlugin,
            unavi_util::UtilPlugin,
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
            unavi_script::ScriptSnapshotSet.after(unavi_agent::AgentMovementSet),
        )
        .add_systems(Startup, icon::set_window_icon);

        // The endpoint key derives from the identity, so the identity plugin
        // has to have built before this runs.
        let secret_key = app
            .world()
            .resource::<unavi_identity::LocalIdentity>()
            .0
            .endpoint()
            .clone();

        app.world_mut().trigger(LoadEndpoint {
            filter: AddrFilter::default(),
            secret_key,
        });
    }
}
