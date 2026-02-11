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
pub struct SqueezeLeftAction;

#[derive(Component, Clone, Copy)]
pub struct SqueezeRightAction;

#[derive(Component)]
pub struct HandLeft;

#[derive(Component)]
pub struct HandRight;

#[derive(Resource)]
pub struct CoreActions {
    pub jump: Entity,
    pub look: Entity,
    pub movement: Entity,
    pub sprint: Entity,
    pub squeeze_left: Entity,
    pub squeeze_right: Entity,
}

#[cfg(not(target_family = "wasm"))]
#[derive(Resource)]
pub struct PoseActions {
    pub left_pose: Entity,
    pub right_pose: Entity,
}

pub(crate) fn setup_actions(mut cmds: Commands) {
    let core_set = cmds.spawn(ActionSet::new("core", "core", 0)).id();

    let movement = cmds
        .spawn((
            MoveAction,
            Action::new("move", "Move", core_set),
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
            OxrBindings::new().bindings(
                OCULUS_TOUCH_PROFILE,
                [
                    "/user/hand/left/input/thumbstick/x",
                    "/user/hand/left/input/thumbstick/y",
                ],
            ),
        ))
        .id();
    let look = cmds
        .spawn((
            LookAction,
            Action::new("look", "Look", core_set),
            Vec2ActionValue::new(),
            GamepadBindings::new().add_stick(
                GamepadBindingSource::RightStickX,
                GamepadBindingSource::RightStickY,
            ),
            MouseBindings::new().delta_motion(),
            #[cfg(not(target_family = "wasm"))]
            OxrBindings::new().bindings(
                OCULUS_TOUCH_PROFILE,
                [
                    "/user/hand/right/input/thumbstick/x",
                    "/user/hand/right/input/thumbstick/y",
                ],
            ),
        ))
        .id();
    let jump = cmds
        .spawn((
            JumpAction,
            Action::new("jump", "Jump", core_set),
            BoolActionValue::new(),
            GamepadBindings::new().bind(GamepadBinding::new(GamepadBindingSource::South)),
            KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::Space)),
            #[cfg(not(target_family = "wasm"))]
            OxrBindings::new().bindings(
                OCULUS_TOUCH_PROFILE,
                ["/user/hand/left/input/primary/click"],
            ),
        ))
        .id();
    let sprint = cmds
        .spawn((
            SprintAction,
            Action::new("sprint", "Sprint", core_set),
            BoolActionValue::new(),
            GamepadBindings::new().bind(GamepadBinding::new(GamepadBindingSource::LeftStickClick)),
            KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::ShiftLeft)),
            #[cfg(not(target_family = "wasm"))]
            OxrBindings::new().bindings(
                OCULUS_TOUCH_PROFILE,
                ["/user/hand/left/input/thumbstick/click"],
            ),
        ))
        .id();
    let squeeze_left = cmds
        .spawn((
            SqueezeLeftAction,
            Action::new("squeeze_l", "Squeeze Left", core_set),
            BoolActionValue::new(),
            #[cfg(not(target_family = "wasm"))]
            OxrBindings::new().bindings(
                OCULUS_TOUCH_PROFILE,
                ["/user/hand/left/input/squeeze/value"],
            ),
        ))
        .id();
    let squeeze_right = cmds
        .spawn((
            SqueezeRightAction,
            Action::new("squeeze_r", "Squeeze Right", core_set),
            BoolActionValue::new(),
            GamepadBindings::new().bind(GamepadBinding::new(GamepadBindingSource::RightTrigger)),
            MouseBindings::new().bind(MouseButtonBinding::new(MouseButton::Left)),
            #[cfg(not(target_family = "wasm"))]
            OxrBindings::new().bindings(
                OCULUS_TOUCH_PROFILE,
                ["/user/hand/right/input/squeeze/value"],
            ),
        ))
        .id();

    cmds.insert_resource(CoreActions {
        jump,
        look,
        movement,
        sprint,
        squeeze_left,
        squeeze_right,
    });

    #[cfg(not(target_family = "wasm"))]
    {
        let pose_set = cmds.spawn(ActionSet::new("pose", "Poses", 0)).id();
        let left_hand = cmds.spawn(HandLeft).id();
        let right_hand = cmds.spawn(HandRight).id();
        let left_pose = cmds
            .spawn((
                Action::new("hand_left_pose", "Left Hand Pose", pose_set),
                OxrBindings::new()
                    .bindings(OCULUS_TOUCH_PROFILE, ["/user/hand/left/input/grip/pose"]),
                AttachSpaceToEntity(left_hand),
                SpaceActionValue::new(),
            ))
            .id();
        let right_pose = cmds
            .spawn((
                Action::new("hand_right_pose", "Right Hand Pose", pose_set),
                OxrBindings::new()
                    .bindings(OCULUS_TOUCH_PROFILE, ["/user/hand/right/input/grip/pose"]),
                AttachSpaceToEntity(right_hand),
                SpaceActionValue::new(),
            ))
            .id();
        cmds.insert_resource(PoseActions {
            left_pose,
            right_pose,
        });
    }
}
