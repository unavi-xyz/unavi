use bevy::prelude::*;

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

    app.add_systems(Startup, unavi_app::icon::set_window_icon);

    app.run();
}
