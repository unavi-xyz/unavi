use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod avatar;
mod player;
mod settings;
mod world;

pub struct StartOptions {
    pub file_path: String,
    pub callback: Option<Box<dyn FnOnce(&mut App)>>,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            file_path: "assets".to_string(),
            callback: None,
        }
    }
}

pub fn start(options: StartOptions) {
    let mut app = App::new();

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
                file_path: options.file_path,
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

    if let Some(callback) = options.callback {
        callback(&mut app);
    }

    app.run();
}
