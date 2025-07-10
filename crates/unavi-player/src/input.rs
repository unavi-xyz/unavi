use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};
use unavi_input::{schminput::prelude::*, JumpAction, LookAction, MoveAction};

use crate::{JumpStrength, PlayerBody, PlayerHead, RealHeight, WalkSpeed};

pub fn apply_head_input(
    look_action: Query<&Vec2ActionValue, With<LookAction>>,
    mut body: Query<&mut Transform, (With<PlayerBody>, Without<PlayerHead>)>,
    mut head: Query<&mut Transform, (With<PlayerHead>, Without<PlayerBody>)>,
    mut target: Local<Vec2>,
    time: Res<Time>,
) {
    let Ok(action) = look_action.single() else {
        return;
    };

    let Ok(mut body_tr) = body.single_mut() else {
        return;
    };

    let Ok(mut head_tr) = head.single_mut() else {
        return;
    };

    let delta = time.delta_secs();
    let sensitivity = 0.15;
    *target += action.any * delta * sensitivity;

    const PITCH_BOUND: f32 = FRAC_PI_2 - 1E-3;

    target.y = target.y.clamp(-PITCH_BOUND, PITCH_BOUND);

    const S: f32 = 0.4;

    let pitch = Quat::from_rotation_x(target.y);
    head_tr.rotation = head_tr.rotation.lerp(pitch, S);

    let yaw = Quat::from_rotation_y(-target.x);
    body_tr.rotation = body_tr.rotation.lerp(yaw, S);
}

pub fn apply_body_input(
    head: Query<&GlobalTransform, With<PlayerHead>>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    mut controller: Query<(&mut TnuaController, &WalkSpeed, &RealHeight, &JumpStrength)>,
) {
    let Ok(head_tr) = head.single() else {
        return;
    };

    let Ok((mut controller, speed, height, jump_height)) = controller.single_mut() else {
        return;
    };

    if let Ok(action) = move_action.single() {
        let input = action.normalize_or_zero();

        let dir_f = head_tr.rotation().mul_vec3(Vec3::NEG_Z);
        let dir_l = head_tr.rotation().mul_vec3(Vec3::X);

        let mut dir = Vec3::ZERO;
        dir += dir_f * input.y;
        dir += dir_l * input.x;

        controller.basis(TnuaBuiltinWalk {
            desired_velocity: dir * speed.0,
            float_height: height.0 * 0.8,
            ..Default::default()
        });
    };

    if let Ok(action) = jump_action.single() {
        if action.any {
            controller.action(TnuaBuiltinJump {
                height: jump_height.0 * height.0,
                ..Default::default()
            });
        }
    };
}
