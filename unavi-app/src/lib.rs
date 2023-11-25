use bevy::{log::LogPlugin, prelude::*};
use bevy_rapier3d::prelude::*;

pub mod avatar;
pub mod networking;
pub mod player;
pub mod settings;
pub mod world;

pub use bevy::app::App;

pub struct UnaviPlugin {
    pub file_path: String,
    pub log_level: tracing::Level,
}

impl Default for UnaviPlugin {
    fn default() -> Self {
        Self {
            file_path: "assets".to_string(),
            log_level: tracing::Level::INFO,
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
                })
                .set(LogPlugin {
                    level: self.log_level,
                    ..default()
                }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            avatar::AvatarPlugin,
            networking::NetworkingPlugin,
            player::PlayerPlugin,
            settings::SettingsPlugin,
            world::WorldPlugin,
            // bevy::diagnostic::LogDiagnosticsPlugin::default(),
            // bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        ));
    }
}
