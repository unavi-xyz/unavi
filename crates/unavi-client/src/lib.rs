use std::{path::PathBuf, sync::LazyLock};

use bevy::{
    asset::io::web::WebAssetPlugin, light::light_consts::lux, prelude::*, window::WindowTheme,
};
use bevy_av1::VideoTargetApp;
use bevy_rich_text3d::Text3dPlugin;
use bitflags::bitflags;
use blake3::Hash;
use directories::ProjectDirs;

pub mod assets;
mod async_commands;
#[cfg(feature = "devtools-network")]
mod devtools;
mod fade;
mod icon;
mod networking;
mod scene;
mod space;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub fn assets_dir() -> PathBuf {
    DIRS.data_local_dir().join("assets")
}

#[must_use]
pub fn models_dir() -> PathBuf {
    assets_dir().join("model")
}

#[must_use]
pub fn images_dir() -> PathBuf {
    assets_dir().join("image")
}

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
    pub initial_space: Option<Hash>,
}

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        assets::copy::copy_assets_to_dirs().expect("failed to copy assets");
        assets::download::download_web_assets().expect("failed to download web assets");

        app.add_plugins((
            DefaultPlugins
                .set(WebAssetPlugin {
                    silence_startup_warning: true,
                })
                .set(AssetPlugin {
                    file_path: assets_dir().to_string_lossy().to_string(),
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
                }),
            avian3d::PhysicsPlugins::default(),
            bevy_av1::VideoPlugin,
            bevy_seedling::SeedlingPlugin::default(),
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
            unavi_script::ScriptPlugin,
            networking::NetworkingPlugin,
            space::SpacePlugin {
                initial_space: self.initial_space,
            },
        ))
        .init_video_target_asset::<StandardMaterial>()
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
