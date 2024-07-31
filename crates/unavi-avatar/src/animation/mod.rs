use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};

pub mod load;
mod mixamo;

use self::load::AvatarAnimationNodes;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum AnimationName {
    Falling,
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

pub fn play_avatar_animations(
    avatars: Query<(&Handle<AnimationGraph>, &AvatarAnimationNodes)>,
    mut commands: Commands,
    mut new_players: Query<(Entity, &mut AnimationPlayer, &Parent), Added<AnimationPlayer>>,
) {
    for (entity, mut player, parent) in new_players.iter_mut() {
        if let Ok((graph, nodes)) = avatars.get(parent.get()) {
            let mut transitions = AnimationTransitions::default();

            let node = nodes.0[&AnimationName::Idle];
            transitions.play(&mut player, node, Duration::ZERO).repeat();

            commands
                .entity(entity)
                .insert((graph.clone(), nodes.clone(), transitions));
        }
    }
}
