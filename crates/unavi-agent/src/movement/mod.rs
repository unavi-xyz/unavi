use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_tnua::prelude::{
    TnuaBuiltinJump,
    TnuaBuiltinWalk,
    TnuaController,
};
use unavi_input::{
    action::{
        Action,
        ActionState,
    },
    config::InputConfig,
};

use crate::{
    AgentRig,
    ControlScheme,
    LocalAgentEntities,
    config::{
        AgentConfig,
        XrMode,
    },
    tracking::{
        TrackedHead,
        TrackedPose,
    },
};

pub mod grounded;
pub mod teleport;
#[cfg(not(target_family = "wasm"))] pub mod xr;

#[derive(Resource, Default)]
pub struct MovementYaw(pub f32);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TargetBodyInput(Vec3);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TargetHeadInput(Vec2);

const PITCH_BOUND: f32 = FRAC_PI_2 - 1.0E-3;

pub fn apply_head_input(
    input: Res<ActionState>,
    config: Res<InputConfig>,
    agents: Query<&LocalAgentEntities>,
    mut rigs: Query<&mut Transform, With<AgentRig>>,
    mut tracked_heads: Query<&mut TrackedPose, With<TrackedHead>>,
    mut target: ResMut<TargetHeadInput>,
    time: Res<Time>,
) {
    const S: f32 = 0.4;

    let tuning = &config.tuning;
    let stick =
        input.axis(Action::Look) * tuning.look_degrees_per_second.to_radians() * time.delta_secs();
    let mouse = input.delta(Action::Look) * tuning.look_sensitivity;

    target.0 += stick + mouse;
    target.y = target.y.clamp(-PITCH_BOUND, PITCH_BOUND);

    for entities in agents.iter() {
        if let Ok(mut rig_transform) = rigs.get_mut(entities.body) {
            let yaw = Quat::from_rotation_y(-target.x);
            rig_transform.rotation = rig_transform.rotation.lerp(yaw, S);
        }

        if let Ok(mut pose) = tracked_heads.get_mut(entities.tracked_head) {
            let target_pose = Quat::from_rotation_x(target.y);
            pose.rotation = pose.rotation.lerp(target_pose, S);
        }
    }
}

pub fn apply_body_input(
    agents: Query<(&LocalAgentEntities, &AgentConfig)>,
    input: Res<ActionState>,
    input_config: Res<InputConfig>,
    rigs: Query<&Transform, With<AgentRig>>,
    mut controllers: Query<&mut TnuaController<ControlScheme>, With<AgentRig>>,
    mut target: ResMut<TargetBodyInput>,
    xr: Res<XrMode>,
    movement_yaw: Res<MovementYaw>,
) {
    const S: f32 = 0.2;

    let raw = input.axis(Action::Move);
    let walk = if raw.length() < input_config.tuning.move_threshold {
        Vec2::ZERO
    } else {
        raw
    };

    for (entities, config) in agents.iter() {
        let Ok(rig_transform) = rigs.get(entities.body) else {
            continue;
        };

        let Ok(mut controller) = controllers.get_mut(entities.body) else {
            continue;
        };

        controller.initiate_action_feeding();

        let forward = if xr.0 {
            Quat::from_rotation_y(movement_yaw.0)
        } else {
            rig_transform.rotation
        };
        let dir_f = forward * Vec3::NEG_Z;
        let dir_l = forward * Vec3::X;

        let mut dir = Vec3::ZERO;
        dir += dir_f * walk.y;
        dir += dir_l * walk.x;

        target.0 = target.lerp(dir, S);

        let multi = if input.pressed(Action::Sprint) {
            config.sprint_multi
        } else {
            1.0
        };

        controller.basis = TnuaBuiltinWalk {
            desired_motion: target.0 * multi,
            ..Default::default()
        };

        if input.pressed(Action::Jump) {
            controller.action(ControlScheme::Jump(TnuaBuiltinJump::default()));
        }
    }
}
