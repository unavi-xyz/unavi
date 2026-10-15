use bevy::{
    animation::ActiveAnimation,
    prelude::*,
};

use super::{
    AnimationName,
    AvatarAnimationNodes,
    locomotion::LocomotionProfile,
    velocity::AverageVelocity,
};
use crate::{
    Avatar,
    Grounded,
};

/// Blend weight for each locomotion clip, indexed by name instead of a
/// `HashMap` since the set of names is fixed and known at compile time.
#[derive(Component, Clone, Copy, Default)]
pub struct AnimationWeights {
    idle:       f32,
    walk:       f32,
    walk_left:  f32,
    walk_right: f32,
    sprint:     f32,
    falling:    f32,
}

impl AnimationWeights {
    const fn get(&self, name: AnimationName) -> f32 {
        match name {
            AnimationName::Idle => self.idle,
            AnimationName::Walk => self.walk,
            AnimationName::WalkLeft => self.walk_left,
            AnimationName::WalkRight => self.walk_right,
            AnimationName::Sprint => self.sprint,
            AnimationName::Falling => self.falling,
        }
    }

    const fn set(&mut self, name: AnimationName, value: f32) {
        match name {
            AnimationName::Idle => self.idle = value,
            AnimationName::Walk => self.walk = value,
            AnimationName::WalkLeft => self.walk_left = value,
            AnimationName::WalkRight => self.walk_right = value,
            AnimationName::Sprint => self.sprint = value,
            AnimationName::Falling => self.falling = value,
        }
    }
}

const BLEND_HALFLIFE_SECS: f32 = 0.1;
const WEIGHT_THRESHOLD: f32 = 0.02;

#[derive(Debug, Clone, Copy, Default)]
struct MotionState {
    /// Signed forward velocity (positive = forward, negative = backward).
    forward_speed: f32,
    /// Signed strafe velocity (positive = left, negative = right).
    strafe_speed:  f32,
    is_grounded:   bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct LocomotionWeights {
    idle:       f32,
    walk:       f32,
    walk_left:  f32,
    walk_right: f32,
    sprint:     f32,
    falling:    f32,
}

fn analyze_motion(velocity: Vec3, transform: &Transform) -> MotionState {
    let dir_forward = transform.rotation.mul_vec3(Vec3::NEG_Z);
    let dir_left = transform.rotation.mul_vec3(Vec3::NEG_X);

    MotionState {
        forward_speed: velocity.dot(dir_forward),
        strafe_speed:  velocity.dot(dir_left),
        is_grounded:   true,
    }
}

fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    ((v - a) / (b - a)).clamp(0.0, 1.0)
}

/// Locomotion weights are normalized to sum to 1.0.
fn calculate_locomotion_weights(
    motion: &MotionState,
    profile: LocomotionProfile,
) -> LocomotionWeights {
    let mut weights = LocomotionWeights::default();

    if !motion.is_grounded {
        weights.falling = 1.0;
        return weights;
    }

    let forward = motion.forward_speed;
    let strafe = motion.strafe_speed;
    let speed = forward.hypot(strafe);

    if speed < profile.walk_start() {
        weights.idle = 1.0;
        return weights;
    }

    let forward_abs = forward.abs();
    let strafe_abs = strafe.abs();
    let total = forward_abs + strafe_abs;

    let raw_strafe = if total > 0.0 { strafe_abs / total } else { 0.0 };
    let strafe_ratio = raw_strafe * raw_strafe; // Square to reduce influence.
    let forward_ratio = 1.0 - strafe_ratio;

    let sprint_blend = inverse_lerp(profile.sprint_start(), profile.sprint_end(), speed);
    let walk_blend = 1.0 - sprint_blend;

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
            weights.set(*name, 0.0);
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
        loco_weights.walk_left,
        alpha,
        player,
        nodes,
        weights,
    );

    apply_weight(
        AnimationName::WalkRight,
        loco_weights.walk_right,
        alpha,
        player,
        nodes,
        weights,
    );

    if let Some(walk) = apply_weight(
        AnimationName::Walk,
        loco_weights.walk,
        alpha,
        player,
        nodes,
        weights,
    ) {
        walk.set_speed(if motion.forward_speed.is_sign_positive() {
            1.0
        } else {
            -1.0
        });
    }

    if let Some(sprint) = apply_weight(
        AnimationName::Sprint,
        loco_weights.sprint,
        alpha,
        player,
        nodes,
        weights,
    ) {
        sprint.set_speed(if motion.forward_speed.is_sign_positive() {
            1.0
        } else {
            -1.0
        });
    }

    apply_weight(
        AnimationName::Falling,
        loco_weights.falling,
        alpha,
        player,
        nodes,
        weights,
    );
}

pub fn play_avatar_animations(
    time: Res<Time>,
    rigs: Query<(&Transform, &Grounded)>,
    avatars: Query<(&AvatarAnimationNodes, &AverageVelocity, &LocomotionProfile), With<Avatar>>,
    mut animation_players: Query<(&mut AnimationWeights, &mut AnimationPlayer, &ChildOf)>,
) {
    let alpha = 1.0 - (-time.delta_secs() / BLEND_HALFLIFE_SECS).exp();

    for (mut weights, mut player, parent) in &mut animation_players {
        let Ok((nodes, avg, profile)) = avatars.get(parent.parent()) else {
            continue;
        };

        let Ok((transform, grounded)) = rigs.get(avg.target) else {
            continue;
        };

        initialize_missing_animations(&mut player, &mut weights, nodes);

        let mut motion = analyze_motion(avg.velocity, transform);
        motion.is_grounded = grounded.0;

        let loco_weights = calculate_locomotion_weights(&motion, *profile);

        apply_locomotion_animations(
            &loco_weights,
            alpha,
            &mut player,
            nodes,
            &mut weights,
            &motion,
        );

        apply_weight(
            AnimationName::Idle,
            loco_weights.idle,
            alpha,
            &mut player,
            nodes,
            &mut weights,
        );
    }
}

/// Blends `weights`' entry for `name` toward `target`, and applies the
/// result to the matching animation node if the avatar's clip set has one.
/// `None` (no node loaded for `name`, e.g. the raw asset was missing that
/// clip) is not an error: the avatar simply never plays that animation.
fn apply_weight<'a>(
    name: AnimationName,
    target: f32,
    alpha: f32,
    player: &'a mut AnimationPlayer,
    nodes: &AvatarAnimationNodes,
    weights: &mut AnimationWeights,
) -> Option<&'a mut ActiveAnimation> {
    let prev = weights.get(name);
    let mut weight = target.mul_add(alpha, prev * (1.0 - alpha));

    if weight < WEIGHT_THRESHOLD {
        weight = 0.0;
    }
    weight = weight.min(1.0);
    weights.set(name, weight);

    let node = *nodes.0.get(&name)?;
    let animation = player.animation_mut(node)?;
    animation.set_weight(weight);
    Some(animation)
}
