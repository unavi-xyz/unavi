use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};
use unavi_input::{JumpAction, LookAction, MoveAction, SprintAction, schminput::prelude::*};
use unavi_portal::teleport::PortalTeleport;

use crate::{
    ControlScheme, PlayerEntities, PlayerRig,
    config::PlayerConfig,
    tracking::{TrackedHead, TrackedPose},
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TargetBodyInput(Vec3);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TargetHeadInput(Vec2);

pub fn handle_player_teleport(
    event: On<PortalTeleport>,
    mut target_body: ResMut<TargetBodyInput>,
    mut target_head: ResMut<TargetHeadInput>,
    mut players: Query<&mut LinearVelocity, With<PlayerRig>>,
) {
    // Only apply to players.
    let Ok(mut velocity) = players.get_mut(event.entity) else {
        return;
    };

    // Update target head input.
    let (mut yaw, pitch, _) = event.delta_rotation.to_euler(EulerRot::YXZ);

    if yaw.is_sign_negative() {
        yaw -= FRAC_PI_2;
    } else {
        yaw += FRAC_PI_2;
    }

    target_head.0.x += yaw;
    target_head.0.y += pitch;

    // Update target body input and velocity.
    target_body.0 = target_body.rotate_y(-yaw);
    velocity.0 = velocity.rotate_y(-yaw);
}

/// Applies mouse/keyboard input to the tracked head pose (desktop mode).
pub fn apply_head_input(
    look_action: Query<&Vec2ActionValue, With<LookAction>>,
    players: Query<&PlayerEntities>,
    mut rigs: Query<&mut Transform, With<PlayerRig>>,
    mut tracked_heads: Query<&mut TrackedPose, With<TrackedHead>>,
    mut target: ResMut<TargetHeadInput>,
    time: Res<Time>,
) {
    const PITCH_BOUND: f32 = FRAC_PI_2 - 1E-3;
    const S: f32 = 0.4;

    let Ok(action) = look_action.single() else {
        return;
    };

    let delta = time.delta_secs();
    let sensitivity = 0.1;
    target.0 += action.any * delta * sensitivity;
    target.y = target.y.clamp(-PITCH_BOUND, PITCH_BOUND);

    for entities in players.iter() {
        if let Ok(mut rig_transform) = rigs.get_mut(entities.body) {
            let yaw = Quat::from_rotation_y(-target.x);
            rig_transform.rotation = rig_transform.rotation.lerp(yaw, S);
        }

        if let Ok(mut pose) = tracked_heads.get_mut(entities.tracked_head) {
            let target_pitch = Quat::from_rotation_x(target.y);
            pose.rotation = pose.rotation.lerp(target_pitch, S);
        }
    }
}

/// Applies movement input to the physics controller (all modes).
pub fn apply_body_input(
    players: Query<(&PlayerEntities, &PlayerConfig)>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    sprint_action: Query<&BoolActionValue, With<SprintAction>>,
    rigs: Query<&Transform, With<PlayerRig>>,
    mut controllers: Query<&mut TnuaController<ControlScheme>, With<PlayerRig>>,
    mut target: ResMut<TargetBodyInput>,
) {
    for (entities, config) in players.iter() {
        let Ok(rig_transform) = rigs.get(entities.body) else {
            continue;
        };

        let Ok(mut controller) = controllers.get_mut(entities.body) else {
            continue;
        };

        if let Ok(action) = move_action.single() {
            const S: f32 = 0.2;

            let input = action.normalize_or_zero();

            let dir_f = rig_transform.rotation.mul_vec3(Vec3::NEG_Z);
            let dir_l = rig_transform.rotation.mul_vec3(Vec3::X);

            let mut dir = Vec3::ZERO;
            dir += dir_f * input.y;
            dir += dir_l * input.x;

            target.0 = target.lerp(dir, S);
        }

        let is_sprinting = sprint_action
            .single()
            .map(|action| action.any)
            .unwrap_or(false);

        let speed = if is_sprinting {
            config.sprint_speed
        } else {
            config.walk_speed
        };

        controller.basis = TnuaBuiltinWalk {
            desired_motion: target.0 * speed,
            ..Default::default()
        };

        if let Ok(action) = jump_action.single()
            && action.any
        {
            controller.action(ControlScheme::Jump(TnuaBuiltinJump::default()));
        }
    }
}
