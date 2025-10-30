use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};
use unavi_input::{JumpAction, LookAction, MoveAction, schminput::prelude::*};

use crate::{
    PlayerRig,
    config::{FLOAT_HEIGHT_OFFSET, PLAYER_RADIUS, PlayerConfig},
    tracking::{TrackedHead, TrackedPose},
};

/// Applies mouse/keyboard input to the tracked head pose (desktop mode).
pub(crate) fn apply_head_input(
    look_action: Query<&Vec2ActionValue, With<LookAction>>,
    players: Query<&crate::PlayerEntities>,
    mut rigs: Query<&mut Transform, With<PlayerRig>>,
    mut tracked_heads: Query<&mut TrackedPose, With<TrackedHead>>,
    mut target: Local<Vec2>,
    time: Res<Time>,
) {
    let Ok(action) = look_action.single() else {
        return;
    };

    let delta = time.delta_secs();
    let sensitivity = 0.1;
    *target += action.any * delta * sensitivity;

    const PITCH_BOUND: f32 = FRAC_PI_2 - 1E-3;
    target.y = target.y.clamp(-PITCH_BOUND, PITCH_BOUND);

    const S: f32 = 0.4;

    for entities in players.iter() {
        if let Ok(mut rig_transform) = rigs.get_mut(entities.rig) {
            let yaw = Quat::from_rotation_y(-target.x);
            rig_transform.rotation = rig_transform.rotation.lerp(yaw, S);
        }

        if let Ok(mut pose) = tracked_heads.get_mut(entities.tracked_head) {
            let target_pitch = Quat::from_rotation_x(target.y);
            pose.rotation = pose.rotation.lerp(target_pitch, S);
        }
    }
}

const CAYOTE_TIME: f32 = 0.1;

/// Applies movement input to the physics controller (all modes).
pub(crate) fn apply_body_input(
    players: Query<(&crate::PlayerEntities, &PlayerConfig)>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    rigs: Query<&Transform, With<PlayerRig>>,
    mut controllers: Query<&mut TnuaController, With<PlayerRig>>,
    mut target: Local<Vec3>,
) {
    for (entities, config) in players.iter() {
        let Ok(rig_transform) = rigs.get(entities.rig) else {
            continue;
        };

        let Ok(mut controller) = controllers.get_mut(entities.rig) else {
            continue;
        };

        if let Ok(action) = move_action.single() {
            let input = action.normalize_or_zero();

            let dir_f = rig_transform.rotation.mul_vec3(Vec3::NEG_Z);
            let dir_l = rig_transform.rotation.mul_vec3(Vec3::X);

            let mut dir = Vec3::ZERO;
            dir += dir_f * input.y;
            dir += dir_l * input.x;

            const S: f32 = 0.2;
            *target = target.lerp(dir, S);
        }

        let radius = config.vrm_radius.unwrap_or(PLAYER_RADIUS);
        let height = config.vrm_height.unwrap_or(config.real_height);
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: *target * config.walk_speed,
            float_height: height / 2.0 + radius + FLOAT_HEIGHT_OFFSET,
            coyote_time: CAYOTE_TIME,
            ..Default::default()
        });

        if let Ok(action) = jump_action.single()
            && action.any
        {
            controller.action(TnuaBuiltinJump {
                height: config.jump_strength * height,
                ..Default::default()
            });
        }
    }
}
