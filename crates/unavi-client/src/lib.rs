use std::{path::PathBuf, sync::LazyLock};

use bevy::{asset::io::web::WebAssetPlugin, prelude::*, window::WindowTheme};
use directories::ProjectDirs;
use dwn::{Dwn, stores::NativeDbStore};

pub mod assets;
mod async_commands;
mod auth;
#[cfg(feature = "devtools-network")]
mod devtools;
mod fade;
mod icon;
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
    assets_dir().join("models")
}

#[must_use]
pub fn images_dir() -> PathBuf {
    assets_dir().join("images")
}

pub fn db_path() -> PathBuf {
    DIRS.data_local_dir().join("data.db")
}

pub struct UnaviPlugin {
    pub in_memory: bool,
    #[cfg(feature = "devtools-network")]
    pub debug_network: bool,
}

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        assets::copy_assets_to_dirs().expect("failed to copy assets");

        let store = if self.in_memory {
            NativeDbStore::new_in_memory()
        } else {
            let path = db_path();
            NativeDbStore::new(path)
        }
        .expect("instantiate native db");
        let dwn = Dwn::from(store);

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
            fade::FadePlugin,
            unavi_input::InputPlugin,
            unavi_player::PlayerPlugin,
            unavi_portal::PortalPlugin,
            unavi_script::ScriptPlugin,
            space::SpacePlugin,
        ));

        #[cfg(feature = "devtools-network")]
        {
            if self.debug_network {
                app.add_plugins(devtools::DevToolsPlugin { enabled: true });
            }
        }

        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(LocalDwn(dwn))
            .init_resource::<auth::LocalActor>()
            .add_observer(auth::handle_login)
            .add_systems(
                Startup,
                (
                    auth::trigger_login,
                    icon::set_window_icon,
                    scene::spawn_lights,
                    scene::spawn_scene,
                ),
            )
            .add_systems(FixedUpdate, async_commands::apply_async_commands);
    }
}

#[derive(Resource)]
struct LocalDwn(pub Dwn);
