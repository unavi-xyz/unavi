use std::sync::LazyLock;

use bevy::prelude::*;
use directories::ProjectDirs;
use dwn::{Dwn, stores::NativeDbStore};

mod auth;
mod icon;
mod scene;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
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

        let store = {
            let mut path = DIRS.data_local_dir().to_path_buf();
            path.push("data.db");
            NativeDbStore::new(path).expect("access native db")
        };
        let dwn = Dwn::from(store);

        app.add_plugins((
            avian3d::PhysicsPlugins::default(),
            unavi_input::InputPlugin,
            unavi_player::PlayerPlugin,
        ))
        .insert_resource(LocalDwn(dwn))
        .add_event::<auth::LoginEvent>()
        .add_observer(auth::handle_login)
        .init_resource::<auth::LocalActor>()
        .add_systems(
            Startup,
            (
                auth::trigger_login,
                icon::set_window_icon,
                scene::spawn_lights,
                scene::spawn_scene,
            ),
        );
    }
}

#[derive(Resource)]
struct LocalDwn(pub Dwn);
