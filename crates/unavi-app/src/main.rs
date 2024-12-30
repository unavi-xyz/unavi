use bevy::prelude::*;
use unavi_app::update::check_for_updates;

fn main() {
    if cfg!(not(debug_assertions)) {
        // Only run in release builds.
        check_for_updates().expect("update");
    }

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

    app.add_systems(
        Startup,
        (
            unavi_app::icon::set_window_icon,
            unavi_app::scene::spawn_lights,
            unavi_app::scene::spawn_scene,
        ),
    );

    app.run();
}
