use bevy::{log::LogPlugin, prelude::*};
use bevy_rapier3d::prelude::*;

pub mod avatar;
mod did;
pub mod networking;
pub mod player;
pub mod settings;
pub mod state;
pub mod world;
// pub mod scripting;

pub use bevy::app::App;

pub struct UnaviPlugin {
    pub assets_dir: String,
    pub debug_frame_time: bool,
    pub debug_physics: bool,
    pub log_level: tracing::Level,
}

impl UnaviPlugin {
    pub fn debug() -> Self {
        Self {
            debug_physics: true,
            log_level: tracing::Level::DEBUG,
            ..default()
        }
    }
}

impl Default for UnaviPlugin {
    fn default() -> Self {
        Self {
            assets_dir: "assets".to_string(),
            debug_frame_time: false,
            debug_physics: false,
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
                    file_path: self.assets_dir.clone(),
                    ..default()
                })
                .set(LogPlugin {
                    level: self.log_level,
                    ..default()
                }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            avatar::AvatarPlugin,
            did::DidPlugin,
            networking::NetworkingPlugin,
            player::PlayerPlugin,
            // scripting::ScriptingPlugin,
            settings::SettingsPlugin,
            world::WorldPlugin,
        ))
        .add_state::<state::AppState>();

        if self.debug_physics {
            app.add_plugins(RapierDebugRenderPlugin::default());
        }

        if self.debug_frame_time {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
        }
    }
}
