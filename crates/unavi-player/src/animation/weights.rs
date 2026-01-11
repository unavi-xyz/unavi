use bevy::{animation::ActiveAnimation, platform::collections::HashMap, prelude::*};

use crate::{
    Grounded, Avatar,
    animation::velocity::AverageVelocity,
    config::{DEFAULT_SPRINT_SPEED, DEFAULT_WALK_SPEED},
};

use super::{AnimationName, AvatarAnimationNodes};

#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct AnimationWeights(pub HashMap<AnimationName, f32>);

/// Target an animation towards a specific weight.
#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct TargetAnimationWeights(pub HashMap<AnimationName, f32>);

// Animation blending constants.
const ALPHA_FACTOR: f32 = 100.0;
const VELOCITY_FACTOR: f32 = 2.0;
const WEIGHT_THRESHOLD: f32 = 0.02;

// Locomotion blend thresholds (in velocity units after VELOCITY_FACTOR).
const WALK_START: f32 = 0.5;
const WALK_END: f32 = DEFAULT_WALK_SPEED * 1.5;
const SPRINT_START: f32 = DEFAULT_WALK_SPEED * 2.0;
const SPRINT_END: f32 = DEFAULT_SPRINT_SPEED * 2.0;

// Falling detection (stub for now).
const FALLING_VELOCITY_THRESHOLD: f32 = 5.0;

/// Analyzed motion state from velocity.
#[derive(Debug, Clone, Copy, Default)]
struct MotionState {
    /// Signed forward velocity (positive = forward, negative = backward).
    forward_speed: f32,
    /// Signed strafe velocity (positive = left, negative = right).
    strafe_speed: f32,
    /// Vertical velocity (for falling detection).
    vertical_speed: f32,
    /// Whether player is grounded (stub for now, always true).
    is_grounded: bool,
}

/// Calculated animation weights for locomotion.
#[derive(Debug, Clone, Copy, Default)]
struct LocomotionWeights {
    idle: f32,
    walk: f32,
    walk_left: f32,
    walk_right: f32,
    sprint: f32,
    falling: f32,
}

/// Analyzes velocity and transform to produce motion state.
fn analyze_motion(velocity: Vec3, transform: &Transform) -> MotionState {
    let dir_forward = transform.rotation.mul_vec3(Vec3::NEG_Z);
    let dir_left = transform.rotation.mul_vec3(Vec3::NEG_X);

    let vel_forward = velocity.dot(dir_forward) * VELOCITY_FACTOR;
    let vel_left = velocity.dot(dir_left) * VELOCITY_FACTOR;
    let vel_vertical = velocity.y;

    MotionState {
        forward_speed: vel_forward,
        strafe_speed: vel_left,
        vertical_speed: vel_vertical,
        is_grounded: true,
    }
}

/// Smooth interpolation helper (inverse lerp clamped to 0-1).
fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    ((v - a) / (b - a)).clamp(0.0, 1.0)
}

/// Calculates locomotion animation weights from motion state.
fn calculate_locomotion_weights(motion: &MotionState) -> LocomotionWeights {
    let mut weights = LocomotionWeights::default();

    // Early return for falling (overrides locomotion when not grounded).
    if !motion.is_grounded {
        weights.falling = (motion.vertical_speed.abs() / FALLING_VELOCITY_THRESHOLD).min(1.0);
        return weights;
    }

    // Strafe animations (left/right walking).
    weights.walk_left = motion.strafe_speed.max(0.0);
    weights.walk_right = motion.strafe_speed.min(0.0).abs();

    let forward_abs = motion.forward_speed.abs();
    let strafe_abs = motion.strafe_speed.abs();

    // Base forward movement weight (before separating walk/sprint).
    let mut forward_weight = forward_abs - strafe_abs;
    forward_weight = forward_weight.max(0.0);

    // Blend between walk and sprint based on speed.
    if forward_abs < WALK_START {
        // Below walk threshold: idle.
        weights.idle = 1.0;
    } else if forward_abs < WALK_END {
        // Walk range: blend from 0 to full walk.
        let walk_blend = inverse_lerp(WALK_START, WALK_END, forward_abs);
        weights.walk = forward_weight * walk_blend;
        weights.idle = 1.0 - walk_blend;
    } else if forward_abs < SPRINT_START {
        // Full walk.
        weights.walk = forward_weight;
    } else if forward_abs < SPRINT_END {
        // Transition from walk to sprint.
        let sprint_blend = inverse_lerp(SPRINT_START, SPRINT_END, forward_abs);
        weights.walk = forward_weight * (1.0 - sprint_blend);
        weights.sprint = forward_weight * sprint_blend;
    } else {
        // Full sprint.
        weights.sprint = forward_weight;
    }

    // Reduce idle by strafe movement.
    weights.idle -= strafe_abs;
    weights.idle = weights.idle.max(0.0);

    weights
}

fn initialize_missing_animations(
    player: &mut AnimationPlayer,
    weights: &mut AnimationWeights,
    nodes: &AvatarAnimationNodes,
) {
    for (name, node) in &nodes.0 {
        if player.animation(*node).is_none() {
            let animation = player.play(*node).repeat();
            animation.set_weight(0.0);
            weights.insert(name.clone(), 0.0);
        }
    }
}

fn apply_locomotion_animations(
    loco_weights: &LocomotionWeights,
    alpha: f32,
    player: &mut AnimationPlayer,
    nodes: &AvatarAnimationNodes,
    weights: &mut AnimationWeights,
    motion: &MotionState,
) {
    apply_weight(
        AnimationName::WalkLeft,
        &mut loco_weights.walk_left.clone(),
        alpha,
        player,
        nodes,
        weights,
    );

    apply_weight(
        AnimationName::WalkRight,
        &mut loco_weights.walk_right.clone(),
        alpha,
        player,
        nodes,
        weights,
    );

    let walk = apply_weight(
        AnimationName::Walk,
        &mut loco_weights.walk.clone(),
        alpha,
        player,
        nodes,
        weights,
    );

    if motion.forward_speed.is_sign_positive() {
        walk.set_speed(1.0);
    } else {
        walk.set_speed(-1.0);
    }

    let sprint = apply_weight(
        AnimationName::Sprint,
        &mut loco_weights.sprint.clone(),
        alpha,
        player,
        nodes,
        weights,
    );

    if motion.forward_speed.is_sign_positive() {
        sprint.set_speed(1.0);
    } else {
        sprint.set_speed(-1.0);
    }

    apply_weight(
        AnimationName::Falling,
        &mut loco_weights.falling.clone(),
        alpha,
        player,
        nodes,
        weights,
    );
}

pub fn play_avatar_animations(
    time: Res<Time>,
    rigs: Query<(&Transform, &Grounded)>,
    avatars: Query<(&AvatarAnimationNodes, &AverageVelocity), With<Avatar>>,
    mut animation_players: Query<(
        &mut AnimationWeights,
        &TargetAnimationWeights,
        &mut AnimationPlayer,
        &ChildOf,
    )>,
) {
    let alpha = (time.delta_secs() * ALPHA_FACTOR).min(0.9);

    for (mut weights, targets, mut player, parent) in &mut animation_players {
        let Ok((nodes, avg)) = avatars.get(parent.parent()) else {
            continue;
        };

        let Some(rig_entity) = avg.target else {
            continue;
        };

        let Ok((transform, grounded)) = rigs.get(rig_entity) else {
            continue;
        };

        let airborn = !grounded.0;

        initialize_missing_animations(&mut player, &mut weights, nodes);

        let mut motion = analyze_motion(avg.velocity, transform);
        motion.is_grounded = !airborn;

        let loco_weights = calculate_locomotion_weights(&motion);

        apply_locomotion_animations(
            &loco_weights,
            alpha,
            &mut player,
            nodes,
            &mut weights,
            &motion,
        );

        // Custom animations (_Other).
        let mut other_weight = 0.0;

        for (weight, value) in weights.clone().iter() {
            if matches!(weight, AnimationName::_Other(_)) {
                other_weight += value;

                let mut target_weight = *targets.get(weight).unwrap_or(&0.0);

                let animation = apply_weight(
                    weight.clone(),
                    &mut target_weight,
                    alpha,
                    &mut player,
                    nodes,
                    &mut weights,
                );
                animation.set_speed(0.01);
            }
        }

        // Idle: use calculated weight from locomotion system.
        let mut idle_weight = loco_weights.idle - other_weight;
        idle_weight = idle_weight.max(0.0);

        apply_weight(
            AnimationName::Idle,
            &mut idle_weight,
            alpha,
            &mut player,
            nodes,
            &mut weights,
        );
    }
}

fn apply_weight<'a>(
    name: AnimationName,
    weight: &mut f32,
    alpha: f32,
    player: &'a mut AnimationPlayer,
    nodes: &AvatarAnimationNodes,
    weights: &mut AnimationWeights,
) -> &'a mut ActiveAnimation {
    let prev = &weights[&name];
    *weight = (*weight).mul_add(1.0 - alpha, prev * alpha);

    if *weight < WEIGHT_THRESHOLD {
        *weight = 0.0;
    }

    *weight = weight.min(1.0);

    let animation = player
        .animation_mut(nodes.0[&name])
        .expect("animation not found");
    animation.set_weight(*weight);
    weights.insert(name, *weight);

    animation
}
