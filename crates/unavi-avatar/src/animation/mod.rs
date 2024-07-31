use bevy::{prelude::*, utils::HashMap};

pub mod load;
mod mixamo;

use crate::AverageVelocity;

use self::load::AvatarAnimationNodes;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum AnimationName {
    Falling,
    #[default]
    Idle,
    Walk,
    WalkLeft,
    WalkRight,
}

/// Defines animation clips for an avatar to use.
#[derive(Component, Clone)]
pub struct AvatarAnimationClips(pub HashMap<AnimationName, AvatarAnimation>);

#[derive(Clone)]
pub struct AvatarAnimation {
    pub clip: Handle<AnimationClip>,
    pub gltf: Handle<Gltf>,
}

pub fn init_animations(
    animation_nodes: Query<&Handle<AnimationGraph>, With<AvatarAnimationNodes>>,
    mut animation_players: Query<(Entity, &Parent), Added<AnimationPlayer>>,
    mut commands: Commands,
) {
    for (entity, parent) in animation_players.iter_mut() {
        if let Ok(graph) = animation_nodes.get(parent.get()) {
            commands.entity(entity).insert(graph.clone());
        } else {
            error!(
                "Failed to initialize animation. Animation nodes not found for animation player {}",
                entity
            );
        }
    }
}

const WEIGHT_THRESHOLD: f32 = 0.02;

pub fn play_avatar_animations(
    mut avatars: Query<(&AvatarAnimationNodes, &AverageVelocity, &Transform)>,
    mut animation_players: Query<(&mut AnimationPlayer, &Parent)>,
) {
    for (mut player, parent) in animation_players.iter_mut() {
        if let Ok((nodes, avg, transform)) = avatars.get_mut(**parent) {
            for node in nodes.0.values() {
                if player.animation(*node).is_none() {
                    let animation = player.play(*node).repeat();
                    animation.set_weight(0.0);
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

            let forward = vel_forward.element_sum();
            let left = vel_left.element_sum();

            // Left walk.
            let mut l_walk_weight = left.max(0.0);

            if l_walk_weight < WEIGHT_THRESHOLD {
                l_walk_weight = 0.0;
            }

            player
                .animation_mut(nodes.0[&AnimationName::WalkLeft])
                .unwrap()
                .set_weight(l_walk_weight);

            // Right walk.
            let mut r_walk_weight = left.min(0.0).abs();

            if r_walk_weight < WEIGHT_THRESHOLD {
                r_walk_weight = 0.0;
            }

            player
                .animation_mut(nodes.0[&AnimationName::WalkRight])
                .unwrap()
                .set_weight(r_walk_weight);

            // Walk.
            let mut walk_weight = forward.abs();

            walk_weight -= left.abs();

            if walk_weight < WEIGHT_THRESHOLD {
                walk_weight = 0.0;
            }

            let walk = player.animation_mut(nodes.0[&AnimationName::Walk]).unwrap();

            if forward.is_sign_positive() {
                walk.set_speed(1.0);
            } else {
                walk.set_speed(-1.0);
            }

            walk.set_weight(walk_weight);

            // Idle.
            let mut idle_weight = 1.0;
            idle_weight -= l_walk_weight;
            idle_weight -= r_walk_weight;
            idle_weight -= walk_weight;

            if idle_weight < WEIGHT_THRESHOLD {
                idle_weight = 0.0;
            }

            let idle = player.animation_mut(nodes.0[&AnimationName::Idle]).unwrap();
            idle.set_weight(idle_weight);
        }
    }
}
