use bevy::prelude::*;

pub mod defaults;
pub mod load;
mod mixamo;
pub mod velocity;
pub mod weights;

pub use load::AvatarAnimationNodes;
use weights::{AnimationWeights, TargetAnimationWeights};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum AnimationName {
    Falling,
    #[default]
    Idle,
    Walk,
    WalkLeft,
    WalkRight,
    _Other(&'static str),
}

pub(crate) fn init_animations(
    animation_nodes: Query<&AnimationGraphHandle, With<AvatarAnimationNodes>>,
    mut animation_players: Query<(Entity, &ChildOf), Added<AnimationPlayer>>,
    mut commands: Commands,
) {
    for (entity, parent) in animation_players.iter_mut() {
        let Ok(graph) = animation_nodes.get(parent.parent()) else {
            continue;
        };

        commands.entity(entity).insert((
            AnimationWeights::default(),
            TargetAnimationWeights::default(),
            graph.clone(),
        ));
    }
}
