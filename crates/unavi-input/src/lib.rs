use bevy::prelude::*;
use schminput::prelude::*;
use schminput_rebinding::{
    DefaultSchminputRebindingPlugins,
    config::{ConfigFilePath, LoadSchminputConfig, SaveSchminputConfig},
};

pub use schminput;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        let config_path = ConfigFilePath::Config {
            app_name: "unavi",
            file_name: "input.toml",
        };

        if config_path.path_buf().unwrap().exists() {
            info!("Loading input config on startup");
            app.add_systems(Startup, load_config);
        } else {
            info!("Saving input config on startup");
            app.add_systems(Startup, save_config);
        }

        app.add_plugins((DefaultSchminputPlugins, DefaultSchminputRebindingPlugins))
            .add_systems(Startup, setup_actions)
            .insert_resource(config_path);
    }
}

fn load_config(mut load: EventWriter<LoadSchminputConfig>) {
    load.write_default();
}

fn save_config(mut save: EventWriter<SaveSchminputConfig>) {
    save.write_default();
}

#[derive(Component, Clone, Copy)]
pub struct MoveAction;

#[derive(Component, Clone, Copy)]
pub struct LookAction;

#[derive(Component, Clone, Copy)]
pub struct JumpAction;

fn setup_actions(mut cmds: Commands) {
    let set = cmds.spawn(ActionSet::new("core", "core", 0)).id();
    cmds.spawn((
        Action::new("move", "Move", set),
        Vec2ActionValue::new(),
        KeyboardBindings::new().add_dpad(
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyA,
            KeyCode::KeyD,
        ),
        GamepadBindings::new().add_stick(
            GamepadBindingSource::LeftStickX,
            GamepadBindingSource::LeftStickX,
        ),
        MoveAction,
    ));
    cmds.spawn((
        Action::new("look", "Look", set),
        Vec2ActionValue::new(),
        MouseBindings::new().delta_motion(),
        GamepadBindings::new().add_stick(
            GamepadBindingSource::RightStickX,
            GamepadBindingSource::RightStickY,
        ),
        LookAction,
    ));
    cmds.spawn((
        Action::new("jump", "Jump", set),
        GamepadBindings::new()
            .bind(GamepadBinding::new(GamepadBindingSource::South).button_just_pressed()),
        KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::Space).just_pressed()),
        BoolActionValue::new(),
        JumpAction,
    ));
}
