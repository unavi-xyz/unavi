use std::{path::PathBuf, sync::LazyLock};

use bevy::prelude::*;
use directories::ProjectDirs;
use dwn::{Dwn, stores::NativeDbStore};

pub mod asset_download;
mod async_commands;
mod auth;
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

pub fn models_dir() -> PathBuf {
    assets_dir().join("models")
}

pub fn images_dir() -> PathBuf {
    assets_dir().join("images")
}

pub fn db_path() -> PathBuf {
    DIRS.data_local_dir().join("data.db")
}

pub struct UnaviPlugin;

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                file_path: assets_dir().to_string_lossy().to_string(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    name: Some("unavi".to_string()),
                    title: "UNAVI".to_string(),
                    ..default()
                }),
                ..default()
            })
            .finish(app);

        let store = {
            let path = db_path();
            NativeDbStore::new(path).expect("access native db")
        };
        let dwn = Dwn::from(store);

        app.add_plugins((
            avian3d::PhysicsPlugins::default(),
            unavi_input::InputPlugin,
            unavi_player::PlayerPlugin,
        ))
        .insert_resource(LocalDwn(dwn))
        .init_resource::<auth::LocalActor>()
        .add_observer(auth::handle_login)
        .add_observer(space::handle_space_add)
        .add_observer(space::handle_connect_info_fetched)
        .add_observer(space::connect::handle_space_connect)
        .add_systems(
            Startup,
            (
                auth::trigger_login,
                icon::set_window_icon,
                scene::spawn_lights,
                scene::spawn_scene,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                async_commands::apply_async_commands,
                space::transform::publish_user_transforms,
            ),
        );
    }
}

#[derive(Resource)]
struct LocalDwn(pub Dwn);
