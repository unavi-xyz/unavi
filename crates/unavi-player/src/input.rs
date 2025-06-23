use bevy::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};
use unavi_input::{JumpAction, LookAction, MoveAction, schminput::prelude::*};

use crate::{PlayerHead, PlayerHeight, PlayerSpeed};

pub fn apply_head_input(
    look_action: Query<&Vec2ActionValue, With<LookAction>>,
    head: Query<&mut Transform, With<PlayerHead>>,
) {
    let Ok(action) = look_action.single() else {
        return;
    };
}

pub fn apply_body_input(
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    mut controller: Query<(&mut TnuaController, &PlayerSpeed, &PlayerHeight)>,
) {
    let Ok((mut controller, speed, height)) = controller.single_mut() else {
        return;
    };

    if let Ok(action) = move_action.single() {
        let mut dir = Vec3::ZERO;
        dir.x += action.x;
        dir.z -= action.y;

        controller.basis(TnuaBuiltinWalk {
            desired_velocity: dir.normalize_or_zero() * speed.0,
            float_height: height.0 / 1.9,
            ..Default::default()
        });
    };

    if let Ok(action) = jump_action.single() {
        if action.any {
            controller.action(TnuaBuiltinJump {
                height: height.0,
                ..Default::default()
            });
        }
    };
}
