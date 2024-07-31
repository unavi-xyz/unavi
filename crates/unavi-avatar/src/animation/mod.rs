use std::time::Duration;

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

#[derive(Component, Clone)]
pub struct AvatarAnimations(pub HashMap<AnimationName, AvatarAnimation>);

#[derive(Clone)]
pub struct AvatarAnimation {
    pub clip: Handle<AnimationClip>,
    pub gltf: Handle<Gltf>,
}

pub fn init_animation_transitions(
    animation_nodes: Query<(&AvatarAnimationNodes, &Handle<AnimationGraph>)>,
    mut animation_players: Query<(Entity, &mut AnimationPlayer, &Parent), Added<AnimationPlayer>>,
    mut commands: Commands,
) {
    for (entity, mut player, parent) in animation_players.iter_mut() {
        if let Ok((nodes, graph)) = animation_nodes.get(parent.get()) {
            let mut transitions = AnimationTransitions::default();

            let animation = nodes.0[&AnimationName::Idle];
            transitions
                .play(&mut player, animation, Duration::ZERO)
                .repeat();

            commands.entity(entity).insert((graph.clone(), transitions));
        } else {
            error!(
                "Failed to initialize AnimationTransitions. Animation nodes not found for animation player {}",
                entity
            );
        }
    }
}

const ANIMATION_DURATION: f32 = 0.0;

pub fn play_target_animation(
    mut avatars: Query<(&AvatarAnimationNodes, &TargetAnimation)>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions, &Parent)>,
) {
    for (mut player, mut transitions, parent) in animation_players.iter_mut() {
        if let Ok((nodes, target)) = avatars.get_mut(parent.get()) {
            let new_animation = nodes.0[&target.name];

            if Some(new_animation) != transitions.get_main_animation() {
                info!("Playing animation: {:?}", target);

                let animation = transitions
                    .play(
                        &mut player,
                        new_animation,
                        Duration::from_secs_f32(ANIMATION_DURATION),
                    )
                    .repeat();

                if target.reverse {
                    animation.set_speed(-1.0);
                }
            }
        } else {
            warn!("Target avatar animations not found");
        }
    }
}

const MAX_IDLE_VELOCITY: f32 = 0.2;

#[derive(Component, Debug, Default)]
pub struct TargetAnimation {
    pub name: AnimationName,
    pub reverse: bool,
}

pub fn play_avatar_animations(mut avatars: Query<(&AverageVelocity, &mut TargetAnimation)>) {
    for (avg, mut current_target) in avatars.iter_mut() {
        let (name, reverse) = if avg.velocity.abs().element_sum() < MAX_IDLE_VELOCITY {
            (AnimationName::Idle, false)
        } else {
            (
                AnimationName::Walk,
                avg.velocity.element_sum().is_sign_positive(),
            )
        };

        current_target.name = name;
        current_target.reverse = reverse;
    }
}
