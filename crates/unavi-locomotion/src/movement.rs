use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};
use unavi_avatar::{AnimationName, TargetAnimationWeights};
use unavi_input::{
    actions::{JumpAction, LookAction, MenuAction, MoveAction, SprintAction},
    schminput::{BoolActionValue, Vec2ActionValue},
};
use unavi_portal::teleport::PortalTeleport;

const SENSITIVITY: f32 = 0.08;

#[cfg(target_family = "wasm")]
fn sensitivity() -> f32 {
    use std::sync::OnceLock;
    static IS_FIREFOX: OnceLock<bool> = OnceLock::new();

    let is_firefox = *IS_FIREFOX.get_or_init(|| {
        web_sys::window()
            .and_then(|w| w.navigator().user_agent().ok())
            .is_some_and(|ua| ua.contains("Firefox"))
    });

    if is_firefox {
        // Firefox reports different pointer lock deltas, requiring higher sensitivity.
        SENSITIVITY * 12.0
    } else {
        // Other browsers need a lower sensitivity than native.
        SENSITIVITY * 0.7
    }
}

#[cfg(not(target_family = "wasm"))]
const fn sensitivity() -> f32 {
    SENSITIVITY
}

use crate::{
    AgentEntities, AgentRig, ControlScheme, LocalAgent,
    config::{AgentConfig, XrMode},
    tracking::{TrackedHead, TrackedPose},
};

/// Forward yaw for movement direction.
/// Desktop: unused (reads rig rotation directly).
/// XR: set from HMD yaw each frame.
#[derive(Resource, Default)]
pub struct MovementYaw(pub f32);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TargetBodyInput(Vec3);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TargetHeadInput(Vec2);

pub fn handle_agent_teleport(
    event: On<PortalTeleport>,
    mut target_body: ResMut<TargetBodyInput>,
    mut target_head: ResMut<TargetHeadInput>,
    mut agents: Query<&mut LinearVelocity, With<AgentRig>>,
) {
    let Ok(mut velocity) = agents.get_mut(event.entity) else {
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
    agents: Query<&AgentEntities>,
    mut rigs: Query<&mut Transform, With<AgentRig>>,
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
    target.0 += action.any * delta * sensitivity();
    target.y = target.y.clamp(-PITCH_BOUND, PITCH_BOUND);

    for entities in agents.iter() {
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

#[derive(Resource, Default)]
pub struct MenuAnimationState(bool);

pub fn apply_menu_animation(
    menu_action: Query<&BoolActionValue, With<MenuAction>>,
    mut animations: Query<(&mut TargetAnimationWeights, &ChildOf)>,
    local_agent: Query<&AgentEntities, With<LocalAgent>>,
    mut menu_state: ResMut<MenuAnimationState>,
    mut prev_state: Local<bool>,
) {
    let Ok(AgentEntities { avatar, .. }) = local_agent.single() else {
        return;
    };

    let Ok(menu_action) = menu_action.single() else {
        return;
    };

    for (mut weights, child_of) in &mut animations {
        if child_of.parent() != *avatar {
            continue;
        }

        let just_pressed = menu_action.any && !*prev_state;
        *prev_state = menu_action.any;

        if just_pressed {
            menu_state.0 = !menu_state.0;
        }

        let menu_weight = if menu_state.0 { 1.0 } else { 0.0 };
        weights.insert(AnimationName::Menu, menu_weight);

        break;
    }
}

const MIN_MENU_MOVEMENT: f32 = 0.1;

/// Applies movement input to the physics controller (all modes).
pub fn apply_body_input(
    agents: Query<(&AgentEntities, &AgentConfig)>,
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
            const MOVE_THRESHOLD: f32 = 0.6; // TODO configurable for stick drift (same for look)
            const S: f32 = 0.2;

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

            if menu_state.0 && raw.element_sum().abs() > MIN_MENU_MOVEMENT {
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
