use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod home;
mod player;

const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

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
            // bevy::diagnostic::LogDiagnosticsPlugin::default(),
            // bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        ))
        .insert_resource(FixedTime::new_from_secs(FIXED_TIMESTEP))
        .run();
}
