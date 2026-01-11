use bevy::{animation::ActiveAnimation, platform::collections::HashMap, prelude::*};

use crate::{
    Avatar, Grounded,
    animation::velocity::AverageVelocity,
    config::{DEFAULT_SPRINT_MULTI, DEFAULT_WALK_SPEED},
};

use super::{AnimationName, AvatarAnimationNodes};

#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct AnimationWeights(pub HashMap<AnimationName, f32>);

/// Target an animation towards a specific weight.
#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct TargetAnimationWeights(pub HashMap<AnimationName, f32>);

// Animation blending constants.
const BLEND_HALFLIFE_SECS: f32 = 0.1;
const WEIGHT_THRESHOLD: f32 = 0.02;

// Locomotion blend thresholds (in m/s).
const WALK_START: f32 = DEFAULT_WALK_SPEED / 4.0;
const SPRINT_START: f32 = DEFAULT_WALK_SPEED * ((DEFAULT_SPRINT_MULTI - 1.0) * 0.5 + 1.0);
const SPRINT_END: f32 = DEFAULT_WALK_SPEED * DEFAULT_SPRINT_MULTI;

/// Analyzed motion state from velocity.
#[derive(Debug, Clone, Copy, Default)]
struct MotionState {
    /// Signed forward velocity (positive = forward, negative = backward).
    forward_speed: f32,
    /// Signed strafe velocity (positive = left, negative = right).
    strafe_speed: f32,
    /// Whether player is grounded.
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

    MotionState {
        forward_speed: velocity.dot(dir_forward),
        strafe_speed: velocity.dot(dir_left),
        is_grounded: true,
    }
}

/// Smooth interpolation helper (inverse lerp clamped to 0-1).
fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    ((v - a) / (b - a)).clamp(0.0, 1.0)
}

/// Calculates locomotion animation weights from motion state.
/// Weights are normalized to sum to 1.0.
fn calculate_locomotion_weights(motion: &MotionState) -> LocomotionWeights {
    let mut weights = LocomotionWeights::default();

    // Falling overrides locomotion.
    if !motion.is_grounded {
        weights.falling = 1.0;
        return weights;
    }

    let forward = motion.forward_speed;
    let strafe = motion.strafe_speed;
    let speed = forward.hypot(strafe);

    // Below threshold = idle.
    if speed < WALK_START {
        weights.idle = 1.0;
        return weights;
    }

    // Directional blend - bias toward forward, strafe is subtle accent.
    let forward_abs = forward.abs();
    let strafe_abs = strafe.abs();
    let total = forward_abs + strafe_abs;

    let raw_strafe = if total > 0.0 { strafe_abs / total } else { 0.0 };
    let strafe_ratio = raw_strafe * raw_strafe; // Square to reduce influence.
    let forward_ratio = 1.0 - strafe_ratio;

    // Speed determines walk vs sprint blend.
    let sprint_blend = inverse_lerp(SPRINT_START, SPRINT_END, speed);
    let walk_blend = 1.0 - sprint_blend;

    // Apply directional ratios.
    weights.walk = forward_ratio * walk_blend;
    weights.sprint = forward_ratio * sprint_blend;

    if strafe > 0.0 {
        weights.walk_left = strafe_ratio;
    } else {
        weights.walk_right = strafe_ratio;
    }

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
    // Exponential blend for snappy ~100ms transitions.
    let alpha = 1.0 - (-time.delta_secs() / BLEND_HALFLIFE_SECS).exp();

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

        initialize_missing_animations(&mut player, &mut weights, nodes);

        let mut motion = analyze_motion(avg.velocity, transform);
        motion.is_grounded = grounded.0;

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

        // Idle weight from locomotion, reduced by custom animations.
        let mut idle_weight = (loco_weights.idle - other_weight).max(0.0);

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
    let prev = weights[&name];
    *weight = weight.mul_add(alpha, prev * (1.0 - alpha));

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
