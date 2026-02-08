use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use schminput::prelude::*;
use schminput_rebinding::DefaultSchminputRebindingPlugins;

#[cfg(not(target_family = "wasm"))]
mod config;

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
            .add_systems(Startup, setup_actions)
            .add_systems(Update, cursor_grab);
    }
}

#[derive(Component, Clone, Copy)]
pub struct MoveAction;

#[derive(Component, Clone, Copy)]
pub struct LookAction;

#[derive(Component, Clone, Copy)]
pub struct JumpAction;

#[derive(Component, Clone, Copy)]
pub struct SprintAction;

#[derive(Component, Clone, Copy)]
pub struct SqueezeAction;

fn setup_actions(mut cmds: Commands) {
    let set = cmds.spawn(ActionSet::new("core", "core", 0)).id();
    cmds.spawn((
        MoveAction,
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
    ));
    cmds.spawn((
        LookAction,
        Action::new("look", "Look", set),
        Vec2ActionValue::new(),
        GamepadBindings::new().add_stick(
            GamepadBindingSource::RightStickX,
            GamepadBindingSource::RightStickY,
        ),
        MouseBindings::new().delta_motion(),
    ));
    cmds.spawn((
        JumpAction,
        Action::new("jump", "Jump", set),
        BoolActionValue::new(),
        GamepadBindings::new().bind(GamepadBinding::new(GamepadBindingSource::South)),
        KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::Space)),
    ));
    cmds.spawn((
        SprintAction,
        Action::new("sprint", "Sprint", set),
        BoolActionValue::new(),
        GamepadBindings::new().bind(GamepadBinding::new(GamepadBindingSource::LeftStickClick)),
        KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::ShiftLeft)),
    ));
    cmds.spawn((
        SqueezeAction,
        Action::new("squeeze", "Squeeze", set),
        BoolActionValue::new(),
        GamepadBindings::new().bind(GamepadBinding::new(GamepadBindingSource::RightTrigger)),
        MouseBindings::new().bind(MouseButtonBinding::new(MouseButton::Left)),
    ));
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CursorGrabState {
    #[default]
    Unlocked,
    Locked,
}

pub fn cursor_grab(
    key: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<CursorGrabState>>,
    mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for mut cursor in &mut windows {
        if mouse.just_pressed(MouseButton::Left) {
            cursor.visible = false;
            cursor.grab_mode = CursorGrabMode::Locked;
            next_state.set(CursorGrabState::Locked);
        }

        if key.just_pressed(KeyCode::Escape) {
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
            next_state.set(CursorGrabState::Unlocked);
        }
    }
}
