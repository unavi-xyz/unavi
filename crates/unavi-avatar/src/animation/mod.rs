use bevy::{animation::ActiveAnimation, prelude::*, utils::HashMap};

pub(crate) mod load;
mod mixamo;

use crate::AverageVelocity;

pub use load::{AvatarAnimationNodes, CreatedAnimationGraph};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum AnimationName {
    Falling,
    #[default]
    Idle,
    Menu,
    Walk,
    WalkLeft,
    WalkRight,
}

#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct AnimationWeights(pub HashMap<AnimationName, f32>);

/// Target an animation towards a specific weight.
#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct TargetAnimationWeights(pub HashMap<AnimationName, f32>);

#[derive(Component, Clone)]
pub struct AvatarAnimationClips(pub HashMap<AnimationName, AvatarAnimation>);

#[derive(Clone)]
pub struct AvatarAnimation {
    pub clip: Handle<AnimationClip>,
    pub gltf: Handle<Gltf>,
}

pub(crate) fn init_animations(
    animation_nodes: Query<&Handle<AnimationGraph>, With<AvatarAnimationNodes>>,
    mut animation_players: Query<(Entity, &Parent), Added<AnimationPlayer>>,
    mut commands: Commands,
) {
    for (entity, parent) in animation_players.iter_mut() {
        if let Ok(graph) = animation_nodes.get(parent.get()) {
            commands.entity(entity).insert((
                AnimationWeights::default(),
                TargetAnimationWeights::default(),
                graph.clone(),
            ));
        } else {
            error!(
                "Failed to initialize animation. Animation nodes not found for animation player {}",
                entity
            );
        }
    }
}

const ALPHA_FACTOR: f32 = 100.0;
const VELOCITY_FACTOR: f32 = 2.0;
const WEIGHT_THRESHOLD: f32 = 0.02;

pub(crate) fn play_avatar_animations(
    time: Res<Time>,
    mut avatars: Query<(&AvatarAnimationNodes, &AverageVelocity, &Transform)>,
    mut animation_players: Query<(
        &mut AnimationWeights,
        &TargetAnimationWeights,
        &mut AnimationPlayer,
        &Parent,
    )>,
) {
    let alpha = (time.delta_seconds() * ALPHA_FACTOR).min(0.9);

    for (mut weights, targets, mut player, parent) in animation_players.iter_mut() {
        if let Ok((nodes, avg, transform)) = avatars.get_mut(**parent) {
            for (name, node) in nodes.0.iter() {
                if player.animation(*node).is_none() {
                    let animation = player.play(*node).repeat();
                    animation.set_weight(0.0);
                    weights.insert(*name, 0.0);
                }
            }

            let dir_forward = transform.rotation.mul_vec3(Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            });
            let dir_left = transform.rotation.mul_vec3(Vec3 {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            });

            let vel_forward = avg.velocity * dir_forward;
            let vel_left = avg.velocity * dir_left;

            let forward = vel_forward.element_sum() * VELOCITY_FACTOR;
            let left = vel_left.element_sum() * VELOCITY_FACTOR;

            // Left walk.
            let mut l_walk_weight = left.max(0.0);

            apply_weight(
                AnimationName::WalkLeft,
                &mut l_walk_weight,
                alpha,
                &mut player,
                nodes,
                &mut weights,
            );

            // Right walk.
            let mut r_walk_weight = left.min(0.0).abs();

            apply_weight(
                AnimationName::WalkRight,
                &mut r_walk_weight,
                alpha,
                &mut player,
                nodes,
                &mut weights,
            );

            // Walk.
            let mut walk_weight = forward.abs();

            walk_weight -= left.abs();
            walk_weight -= l_walk_weight;
            walk_weight -= r_walk_weight;

            let walk = apply_weight(
                AnimationName::Walk,
                &mut walk_weight,
                alpha,
                &mut player,
                nodes,
                &mut weights,
            );

            if forward.is_sign_positive() {
                walk.set_speed(1.0);
            } else {
                walk.set_speed(-1.0);
            }

            // Menu.
            let menu_weight = weights[&AnimationName::Menu];
            let mut target_menu_weight = *targets.get(&AnimationName::Menu).unwrap_or(&0.0);

            apply_weight(
                AnimationName::Menu,
                &mut target_menu_weight,
                alpha,
                &mut player,
                nodes,
                &mut weights,
            );

            // Idle.
            let mut idle_weight = 1.0;
            idle_weight -= l_walk_weight;
            idle_weight -= r_walk_weight;
            idle_weight -= walk_weight;
            idle_weight -= menu_weight;

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
    *weight = *weight * (1.0 - alpha) + prev * alpha;

    if *weight < WEIGHT_THRESHOLD {
        *weight = 0.0;
    }

    *weight = weight.min(1.0);

    let animation = player.animation_mut(nodes.0[&name]).unwrap();
    animation.set_weight(*weight);
    weights.insert(name, *weight);

    animation
}
