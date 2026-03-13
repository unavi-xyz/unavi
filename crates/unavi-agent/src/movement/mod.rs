use std::f32::consts::{FRAC_PI_2, FRAC_PI_6, FRAC_PI_8};

use bevy::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};
use unavi_input::{
    actions::{JumpAction, LookAction, MoveAction, SprintAction},
    schminput::{BoolActionValue, Vec2ActionValue},
};

use crate::{
    AgentRig, ControlScheme, LocalAgentEntities,
    config::{AgentConfig, XrMode},
    tracking::{TrackedHead, TrackedPose},
};

pub mod grounded;
pub mod menu;
mod sensitivity;
pub mod teleport;
#[cfg(not(target_family = "wasm"))]
pub mod xr;

#[derive(Resource, Default)]
pub struct MovementYaw(pub f32);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TargetBodyInput(Vec3);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TargetHeadInput(Vec2);

#[derive(Resource, Default)]
pub struct MenuAnimationState(bool);

pub fn apply_head_input(
    look_action: Query<&Vec2ActionValue, With<LookAction>>,
    agents: Query<&LocalAgentEntities>,
    mut rigs: Query<&mut Transform, With<AgentRig>>,
    mut tracked_heads: Query<&mut TrackedPose, With<TrackedHead>>,
    mut target: ResMut<TargetHeadInput>,
    menu_state: ResMut<MenuAnimationState>,
    time: Res<Time>,
) {
    const PITCH_BOUND: f32 = FRAC_PI_2 - 1E-3;
    const S: f32 = 0.4;

    let Ok(action) = look_action.single() else {
        return;
    };

    let delta = time.delta_secs();
    target.0 += action.any * delta * sensitivity::sensitivity();

    if menu_state.0 {
        let menu_pitch_bound_h = PITCH_BOUND - FRAC_PI_6;
        let menu_pitch_bound_l = -PITCH_BOUND - FRAC_PI_8;
        target.y = target.y.clamp(menu_pitch_bound_l, menu_pitch_bound_h);
        target.x = target.x.clamp(-menu::MENU_YAW_BOUND, menu::MENU_YAW_BOUND);
    } else {
        target.y = target.y.clamp(-PITCH_BOUND, PITCH_BOUND);
    }

    for entities in agents.iter() {
        // When menu locked, rotate head sideways.
        // Otherwise, rotate full body.
        if !menu_state.0
            && let Ok(mut rig_transform) = rigs.get_mut(entities.body)
        {
            let yaw = Quat::from_rotation_y(-target.x);
            rig_transform.rotation = rig_transform.rotation.lerp(yaw, S);
        }

        if let Ok(mut pose) = tracked_heads.get_mut(entities.tracked_head) {
            let mut target_pitch = Quat::from_rotation_x(target.y);

            if menu_state.0 {
                let yaw = Quat::from_rotation_y(-target.x);
                target_pitch += yaw;
            }

            pose.rotation = pose.rotation.lerp(target_pitch, S);
        }
    }
}

const MOVE_THRESHOLD: f32 = 0.6; // TODO configurable for stick drift (same for look)
const S: f32 = 0.2;

pub fn apply_body_input(
    agents: Query<(&LocalAgentEntities, &AgentConfig)>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    sprint_action: Query<&BoolActionValue, With<SprintAction>>,
    rigs: Query<&Transform, With<AgentRig>>,
    mut controllers: Query<&mut TnuaController<ControlScheme>, With<AgentRig>>,
    mut target: ResMut<TargetBodyInput>,
    xr: Res<XrMode>,
    movement_yaw: Res<MovementYaw>,
    mut menu_state: ResMut<MenuAnimationState>,
) {
    for (entities, config) in agents.iter() {
        let Ok(rig_transform) = rigs.get(entities.body) else {
            continue;
        };

        let Ok(mut controller) = controllers.get_mut(entities.body) else {
            continue;
        };

        controller.initiate_action_feeding();

        if let Ok(action) = move_action.single() {
            let raw = action.any;
            let input = if raw.length() < MOVE_THRESHOLD {
                Vec2::ZERO
            } else {
                raw.normalize_or_zero()
            };

            let forward = if xr.0 {
                Quat::from_rotation_y(movement_yaw.0)
            } else {
                rig_transform.rotation
            };
            let dir_f = forward * Vec3::NEG_Z;
            let dir_l = forward * Vec3::X;

            let mut dir = Vec3::ZERO;
            dir += dir_f * input.y;
            dir += dir_l * input.x;

            target.0 = target.lerp(dir, S);

            if menu_state.0 && raw.element_sum().abs() > menu::MIN_MENU_MOVEMENT {
                menu_state.0 = false;
            }
        }

        let is_sprinting = sprint_action
            .single()
            .map(|action| action.any)
            .unwrap_or(false);

        let multi = if is_sprinting {
            config.sprint_multi
        } else {
            1.0
        };

        controller.basis = TnuaBuiltinWalk {
            desired_motion: target.0 * multi,
            ..Default::default()
        };

        if let Ok(action) = jump_action.single()
            && action.any
        {
            controller.action(ControlScheme::Jump(TnuaBuiltinJump::default()));
        }
    }
}
