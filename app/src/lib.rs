use bevy::prelude::*;

mod demo;

pub fn start() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            demo::DemoPlugin,
        ))
        .run();
}
