use bevy::{
    light::light_consts::lux,
    log::LogPlugin,
    prelude::*,
    window::WindowTheme,
};
use bevy_iroh::endpoint::LoadEndpoint;
use bitflags::bitflags;
use iroh::endpoint_info::AddrFilter;
use tracing::Level;

mod camera;
mod devtools;
mod fade;
mod grab;
mod icon;
mod scene;

#[cfg(not(target_family = "wasm"))] mod assets;
#[cfg(not(target_family = "wasm"))] mod xr;

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct DebugFlags: u8 {
        const FPS       = 0b1000;
        const INSPECTOR = 0b0100;
        const NETWORK   = 0b0010;
        const PHYSICS   = 0b0001;
    }
}

pub struct UnaviPlugin {
    pub debug:     DebugFlags,
    pub in_memory: bool,
    pub log_level: Level,
    pub xr:        bool,
}

const DISABLED_LOGS: &[&str] = &[
    "cranelift_codegen",
    "loro_internal",
    "offset_allocator",
    "wasmtime_internal_cranelift",
];

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_family = "wasm"))]
        {
            assets::copy::copy_assets_to_dirs().expect("failed to copy assets");
            assets::download::download_web_assets().expect("failed to download web assets");
        }

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
                    ..default()
                }),
                ..default()
            });

        cfg_select! {
            target_family = "wasm" => {
                let default_plugins = default_plugins.set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                });
                app.add_plugins(default_plugins);
            }
            _ => {
                let default_plugins = default_plugins.set(AssetPlugin {
                    file_path: assets::assets_dir().to_string_lossy().to_string(),
                    ..default()
                })
                .disable::<bevy::asset::io::web::WebAssetPlugin>();
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

        #[cfg(feature = "devtools-bevy")]
        {
            if self.debug.contains(DebugFlags::FPS) {
                app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());
            }
            if self.debug.contains(DebugFlags::PHYSICS) {
                app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin);
            }
        }

        app.add_plugins((
            avian3d::PhysicsPlugins::default(),
            bevy_hsd::HsdPlugin,
            bevy_iroh::IrohPlugin,
            bevy_wds::WdsPlugin,
            unavi_agent::AgentPlugin,
            unavi_avatar::AvatarPlugin,
            unavi_identity::IdentityPlugin,
            unavi_input::InputPlugin,
            unavi_portal::PortalPlugin,
            unavi_script::ScriptPlugin,
            unavi_space::SpacePlugin,
            unavi_util::UtilPlugin,
        ))
        .add_plugins((
            camera::CameraPlugin,
            devtools::DevToolsPlugin {
                inspector: self.debug.contains(DebugFlags::INSPECTOR),
                network:   self.debug.contains(DebugFlags::NETWORK),
            },
            fade::FadePlugin,
            grab::GrabPlugin,
            scene::ScenePlugin,
        ))
        .insert_resource(GlobalAmbientLight {
            brightness: lux::OVERCAST_DAY,
            ..default()
        })
        .add_systems(Startup, icon::set_window_icon);

        app.world_mut().trigger(LoadEndpoint {
            filter:                                                          AddrFilter::default(),
            #[cfg(all(feature = "mdns", not(target_family = "wasm")))]
            mdns:                                                            true,
        });
    }
}
