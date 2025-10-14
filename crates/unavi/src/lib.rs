use std::sync::LazyLock;

use bevy::prelude::*;
use directories::ProjectDirs;

mod auth;
mod icon;
mod scene;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi").expect("project dirs");
    std::fs::create_dir_all(dirs.data_dir()).expect("data dir");
    dirs
});

pub struct UnaviPlugin;

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        DefaultPlugins
            .build()
            .set(WindowPlugin {
                primary_window: Some(Window {
                    name: Some("unavi".to_string()),
                    title: "UNAVI".to_string(),
                    ..default()
                }),
                ..default()
            })
            .finish(app);

        app.add_plugins((
            avian3d::PhysicsPlugins::default(),
            unavi_input::InputPlugin,
            unavi_player::PlayerPlugin,
        ))
        .add_systems(
            Startup,
            (
                auth::trigger_login,
                icon::set_window_icon,
                scene::spawn_lights,
                scene::spawn_scene,
            ),
        )
        .add_observer(auth::handle_login);
    }
}
