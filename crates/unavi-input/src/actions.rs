use bevy::prelude::*;
use schminput::prelude::*;

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

pub(crate) fn setup_actions(mut cmds: Commands) {
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
            GamepadBindingSource::LeftStickY,
        ),
        #[cfg(not(target_family = "wasm"))]
        OxrBindings::new().bindings(OCULUS_TOUCH_PROFILE, ["/user/hand/left/input/thumbstick"]),
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
        #[cfg(not(target_family = "wasm"))]
        OxrBindings::new().bindings(OCULUS_TOUCH_PROFILE, ["/user/hand/right/input/thumbstick"]),
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
