use bevy::prelude::*;
use schminput::prelude::*;
use schminput_rebinding::DefaultSchminputRebindingPlugins;

pub use schminput;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultSchminputPlugins, DefaultSchminputRebindingPlugins));
        app.add_systems(Startup, setup_actions);
    }
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
