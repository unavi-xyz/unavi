//! Turns input into rig motion: look (head/body yaw) and move (Tnua's
//! desired motion), smoothed at a fixed rate independent of frame rate.

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
        InputMode,
    },
};

pub mod grounded;
pub mod teleport;
#[cfg(not(target_family = "wasm"))] pub mod xr;

/// Reference yaw thumbstick-relative movement turns around, in XR (where the
/// rig's own rotation does not track the headset). Written once per frame by
/// either the XR turn systems or, on desktop, left at zero (the rig's own
/// transform is used instead; see [`apply_body_input`]).
#[derive(Component, Default)]
pub struct MovementYaw(pub f32);

/// The rig's smoothed move intent, in rig-local space.
#[derive(Component, Default, Deref, DerefMut)]
pub struct TargetBodyInput(Vec3);

/// The rig's smoothed look intent: yaw in `x`, pitch in `y`.
#[derive(Component, Default, Deref, DerefMut)]
pub struct TargetHeadInput(Vec2);

const PITCH_BOUND: f32 = FRAC_PI_2 - 1.0E-3;

/// Time constant for head-look smoothing, chosen to feel like the previous
/// frame-locked `lerp(0.4)` did at 60 Hz.
const LOOK_SMOOTHING_TAU: f32 = 0.033;
/// Time constant for body-movement smoothing, chosen to feel like the
/// previous frame-locked `lerp(0.2)` did at 60 Hz.
const MOVE_SMOOTHING_TAU: f32 = 0.075;

/// The fraction of the distance to a moving target covered in `dt` seconds
/// of exponential smoothing with time constant `tau`. Unlike a fixed
/// per-frame `lerp` factor, applying this twice over `dt/2` each gives the
/// same result as once over `dt`, so the feel does not depend on frame rate.
#[must_use]
fn smoothing_alpha(dt: f32, tau: f32) -> f32 {
    -(-dt / tau).exp() + 1.0
}

pub fn apply_head_input(
    input: Res<ActionState>,
    config: Res<InputConfig>,
    time: Res<Time>,
    agents: Query<&LocalAgentEntities>,
    mut rig_targets: Query<&mut TargetHeadInput, With<AgentRig>>,
    mut transforms: Query<&mut Transform>,
) {
    let tuning = &config.tuning;
    let stick =
        input.axis(Action::Look) * tuning.look_degrees_per_second.to_radians() * time.delta_secs();
    let mouse = input.delta(Action::Look) * tuning.look_sensitivity;
    let alpha = smoothing_alpha(time.delta_secs(), LOOK_SMOOTHING_TAU);

    for entities in agents.iter() {
        let Ok(mut target) = rig_targets.get_mut(entities.body) else {
            continue;
        };

        target.0 += stick + mouse;
        target.y = target.y.clamp(-PITCH_BOUND, PITCH_BOUND);

        if let Ok(mut rig_transform) = transforms.get_mut(entities.body) {
            let yaw = Quat::from_rotation_y(-target.x);
            rig_transform.rotation = rig_transform.rotation.lerp(yaw, alpha);
        }

        if let Ok(mut head_transform) = transforms.get_mut(entities.tracked_head) {
            let target_pose = Quat::from_rotation_x(target.y);
            head_transform.rotation = head_transform.rotation.lerp(target_pose, alpha);
        }
    }
}

pub fn apply_body_input(
    agents: Query<(&LocalAgentEntities, &AgentConfig)>,
    input: Res<ActionState>,
    input_config: Res<InputConfig>,
    time: Res<Time>,
    xr: Res<InputMode>,
    mut rigs: Query<
        (
            &Transform,
            &mut TargetBodyInput,
            &MovementYaw,
            &mut TnuaController<ControlScheme>,
        ),
        With<AgentRig>,
    >,
) {
    let alpha = smoothing_alpha(time.delta_secs(), MOVE_SMOOTHING_TAU);

    let raw = input.axis(Action::Move);
    let walk = if raw.length() < input_config.tuning.move_threshold {
        Vec2::ZERO
    } else {
        raw
    };

    for (entities, config) in agents.iter() {
        let Ok((rig_transform, mut target, movement_yaw, mut controller)) =
            rigs.get_mut(entities.body)
        else {
            continue;
        };

        controller.initiate_action_feeding();

        let forward = if xr.is_xr() {
            Quat::from_rotation_y(movement_yaw.0)
        } else {
            rig_transform.rotation
        };
        let dir_f = forward * Vec3::NEG_Z;
        let dir_l = forward * Vec3::X;

        let mut dir = Vec3::ZERO;
        dir += dir_f * walk.y;
        dir += dir_l * walk.x;

        target.0 = target.lerp(dir, alpha);

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

#[cfg(test)]
mod tests {
    use super::smoothing_alpha;

    /// Two half-steps of smoothing must land in the same place as one full
    /// step, so the rig turns and accelerates at the same rate regardless of
    /// how the frame time is split.
    #[test]
    fn smoothing_is_independent_of_how_dt_is_split() {
        const TAU: f32 = 0.05;

        let one_step = smoothing_alpha(1.0 / 30.0, TAU);
        let half = smoothing_alpha(1.0 / 60.0, TAU);

        // Blending a value toward a fixed target with alpha `a` leaves
        // `1 - a` of the gap; applying it twice leaves `(1 - a)^2`.
        let two_steps_remaining = (1.0 - half) * (1.0 - half);
        let one_step_remaining = 1.0 - one_step;

        assert!(
            (two_steps_remaining - one_step_remaining).abs() < 1.0e-6,
            "two 1/60s steps ({two_steps_remaining}) should match one 1/30s step \
             ({one_step_remaining})"
        );
    }

    #[test]
    fn zero_dt_leaves_the_target_unmoved() {
        assert_eq!(smoothing_alpha(0.0, 0.05), 0.0);
    }

    #[test]
    fn alpha_approaches_one_as_dt_grows() {
        assert!(smoothing_alpha(10.0, 0.05) > 0.999);
    }
}
