use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod avatar;
mod did;
pub mod networking;
pub mod player;
pub mod scripting;
pub mod settings;
pub mod state;
pub mod world;

pub use bevy::app::App;

#[derive(Debug, Default)]
pub struct UnaviPlugin {
    pub debug_frame_time: bool,
    pub debug_physics: bool,
}

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            avatar::AvatarPlugin,
            did::DidPlugin,
            networking::NetworkingPlugin,
            player::PlayerPlugin,
            scripting::ScriptingPlugin,
            settings::SettingsPlugin,
            world::WorldPlugin,
        ))
        .init_state::<state::AppState>();

        if self.debug_physics {
            app.add_plugins(RapierDebugRenderPlugin::default());
        }

        if self.debug_frame_time {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
        }
    }
}
