#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;
#[cfg(not(target_family = "wasm"))]
use std::sync::LazyLock;

use bevy::{
    asset::io::web::WebAssetPlugin, light::light_consts::lux, log::LogPlugin, prelude::*,
    window::WindowTheme,
};
use bevy_rich_text3d::Text3dPlugin;
use bitflags::bitflags;
use blake3::Hash;
#[cfg(not(target_family = "wasm"))]
use directories::ProjectDirs;
use tracing::Level;

#[cfg(not(target_family = "wasm"))]
mod assets;
mod async_commands;
#[cfg(feature = "devtools-network")]
mod devtools;
mod fade;
mod icon;
mod networking;
mod scene;
mod space;
mod util;

#[cfg(not(target_family = "wasm"))]
pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

#[cfg(not(target_family = "wasm"))]
pub fn assets_dir() -> PathBuf {
    DIRS.data_local_dir().join("assets")
}

#[cfg(not(target_family = "wasm"))]
#[must_use]
pub fn models_dir() -> PathBuf {
    assets_dir().join("model")
}

#[cfg(not(target_family = "wasm"))]
#[must_use]
pub fn images_dir() -> PathBuf {
    assets_dir().join("image")
}

#[cfg(not(target_family = "wasm"))]
pub fn db_path() -> PathBuf {
    DIRS.data_local_dir().join("data.db")
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct DebugFlags: u8 {
        const FPS     = 0b0001;
        const NETWORK = 0b0010;
        const PHYSICS = 0b0100;
    }
}

pub struct UnaviPlugin {
    pub debug: DebugFlags,
    pub in_memory: bool,
    pub initial_space: Option<Hash>,
    pub log_level: Level,
}

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_family = "wasm"))]
        {
            assets::copy::copy_assets_to_dirs().expect("failed to copy assets");
            assets::download::download_web_assets().expect("failed to download web assets");
        }

        #[cfg(not(target_family = "wasm"))]
        let asset_plugin = AssetPlugin {
            file_path: assets_dir().to_string_lossy().to_string(),
            ..default()
        };
        #[cfg(target_family = "wasm")]
        let asset_plugin = AssetPlugin::default();

        app.add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    filter: format!(
                        "{},loro_internal=off,offset_allocator=off",
                        bevy::log::DEFAULT_FILTER
                    ),
                    level: self.log_level,
                    ..default()
                })
                .set(WebAssetPlugin {
                    silence_startup_warning: true,
                })
                .set(asset_plugin)
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        name: Some("unavi".to_string()),
                        title: "UNAVI".to_string(),
                        window_theme: Some(WindowTheme::Dark),
                        ..default()
                    }),
                    ..default()
                }),
            avian3d::PhysicsPlugins::default(),
            Text3dPlugin {
                asynchronous_load: true,
                load_system_fonts: true,
                sync_scale_factor_with_main_window: true,
                ..default()
            },
            fade::FadePlugin,
            unavi_input::InputPlugin,
            unavi_player::PlayerPlugin,
            unavi_portal::PortalPlugin,
            #[cfg(not(target_family = "wasm"))]
            unavi_script::ScriptPlugin,
            networking::NetworkingPlugin {
                wds_in_memory: self.in_memory,
            },
            space::SpacePlugin {
                initial_space: self.initial_space,
            },
        ))
        .insert_resource(AmbientLight {
            brightness: lux::OVERCAST_DAY,
            ..default()
        });

        #[cfg(feature = "devtools-bevy")]
        {
            if self.debug.contains(DebugFlags::FPS) {
                app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());
            }
            if self.debug.contains(DebugFlags::PHYSICS) {
                app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin);
            }
        }

        #[cfg(feature = "devtools-network")]
        {
            if self.debug.contains(DebugFlags::NETWORK) {
                app.add_plugins(devtools::DevToolsPlugin { enabled: true });
            }
        }

        app.insert_resource(ClearColor(Color::BLACK))
            .add_systems(
                Startup,
                (
                    icon::set_window_icon,
                    scene::spawn_lights,
                    scene::spawn_scene,
                ),
            )
            .add_systems(FixedUpdate, async_commands::apply_async_commands);
    }
}
