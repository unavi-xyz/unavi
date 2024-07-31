use avian3d::parry::na::RealField;
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

const MAX_IDLE_VELOCITY: f32 = 0.3;

// Rate at which animations transition.
const IDLE_RATE: f32 = 2.0;
const WALK_RATE: f32 = 2.0;

pub fn play_avatar_animations(
    time: Res<Time>,
    mut avatars: Query<(&AvatarAnimationNodes, &AverageVelocity, &Transform)>,
    mut animation_players: Query<(&mut AnimationPlayer, &Parent)>,
) {
    let delta = time.delta_seconds();

    for (mut player, parent) in animation_players.iter_mut() {
        if let Ok((nodes, avg, transform)) = avatars.get_mut(**parent) {
            for node in nodes.0.values() {
                if player.animation(*node).is_none() {
                    let animation = player.play(*node).repeat();
                    animation.set_weight(0.0);
                }
            }

            // Idle.
            let idle = player.animation_mut(nodes.0[&AnimationName::Idle]).unwrap();
            let idle_weight = idle.weight();
            // info!("idle: {}", idle_weight);

            if avg.velocity.abs().element_sum() < MAX_IDLE_VELOCITY {
                if idle_weight < 1.0 {
                    idle.set_weight(idle_weight + (IDLE_RATE * delta));
                }
            } else if idle_weight > 0.0 {
                idle.set_weight(idle_weight - (IDLE_RATE * delta));
            }

            // Walk.
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

            let walk = player.animation_mut(nodes.0[&AnimationName::Walk]).unwrap();
            let walk_weight = walk.weight();
            // info!("walk: {}", walk_weight);

            let forward_bool = forward.is_sign_positive();

            if forward_bool {
                walk.set_speed(1.0);
            } else {
                walk.set_speed(-1.0);
            }

            if forward.abs() > MAX_IDLE_VELOCITY {
                if walk_weight < 1.0 {
                    walk.set_weight(walk_weight + (WALK_RATE * delta));
                }
            } else if walk_weight > 0.0 {
                walk.set_weight(walk_weight - (WALK_RATE * delta));
            }

            // Left walk.
            let l_walk = player
                .animation_mut(nodes.0[&AnimationName::WalkLeft])
                .unwrap();
            let l_walk_weight = l_walk.weight();
            // info!("l walk: {}", l_walk_weight);

            if left > MAX_IDLE_VELOCITY {
                if l_walk_weight < 1.0 {
                    l_walk.set_weight(l_walk_weight + (WALK_RATE * delta));
                }
            } else if l_walk_weight > 0.0 {
                l_walk.set_weight(l_walk_weight - (WALK_RATE * delta));
            }

            // Right walk.
            let r_walk = player
                .animation_mut(nodes.0[&AnimationName::WalkRight])
                .unwrap();
            let r_walk_weight = r_walk.weight();
            // info!("r walk: {}", r_walk_weight);

            if (-left) > MAX_IDLE_VELOCITY {
                if r_walk_weight < 1.0 {
                    r_walk.set_weight(r_walk_weight + (WALK_RATE * delta));
                }
            } else if r_walk_weight > 0.0 {
                r_walk.set_weight(r_walk_weight - (WALK_RATE * delta));
            }
        }
    }
}
