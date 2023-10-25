use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod home;
mod player;

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
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            home::HomePlugin,
            player::PlayerPlugin,
        ))
        .run();
}
