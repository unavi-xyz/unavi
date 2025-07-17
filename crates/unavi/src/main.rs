use avian3d::PhysicsPlugins;
use bevy::prelude::*;
use unavi_input::InputPlugin;
use unavi_player::PlayerPlugin;

fn main() {
    let mut app = App::new();

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
        .finish(&mut app);

    app.add_plugins((PhysicsPlugins::default(), InputPlugin, PlayerPlugin));

    app.add_systems(
        Startup,
        (
            unavi::icon::set_window_icon,
            unavi::scene::spawn_lights,
            unavi::scene::spawn_scene,
        ),
    );

    app.run();
}
