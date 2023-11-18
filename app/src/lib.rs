use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod avatar;
mod player;
mod settings;
mod world;

pub use bevy::app::App;

pub struct UnaviPlugin {
    pub file_path: String,
}

impl Default for UnaviPlugin {
    fn default() -> Self {
        Self {
            file_path: "assets".to_string(),
        }
    }
}

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: self.file_path.clone(),
                    ..default()
                }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            avatar::AvatarPlugin,
            world::WorldPlugin,
            settings::SettingsPlugin,
            player::PlayerPlugin,
            // bevy::diagnostic::LogDiagnosticsPlugin::default(),
            // bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        ));
    }
}
