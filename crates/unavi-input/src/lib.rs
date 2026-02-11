use bevy::prelude::*;
use schminput::DefaultSchminputPlugins;
use schminput_rebinding::DefaultSchminputRebindingPlugins;

pub mod actions;
#[cfg(not(target_family = "wasm"))]
mod config;
pub mod cursor_lock;
pub mod raycast;

pub use schminput;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_family = "wasm"))]
        {
            use schminput_rebinding::config::ConfigFilePath;

            let config_path = ConfigFilePath::Config {
                app_name: "unavi",
                file_name: "input.toml",
            };

            if config_path
                .path_buf()
                .expect("failed to get config path")
                .exists()
            {
                info!("Loading input config on startup");
                app.add_systems(Startup, config::load_config);
            } else {
                info!("Saving input config on startup");
                app.add_systems(Startup, config::save_config);
            }

            app.insert_resource(config_path);
        }

        app.add_plugins((DefaultSchminputPlugins, DefaultSchminputRebindingPlugins))
            .init_state::<cursor_lock::CursorGrabState>()
            .add_systems(Startup, actions::setup_actions)
            .add_systems(Update, cursor_lock::cursor_grab)
            .add_systems(FixedUpdate, raycast::read_raycast_input);
    }
}

#[derive(EntityEvent)]
pub struct SqueezeDown(pub Entity);

#[derive(EntityEvent)]
pub struct SqueezeUp(pub Entity);
