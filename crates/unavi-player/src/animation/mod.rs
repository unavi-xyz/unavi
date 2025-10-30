use bevy::prelude::*;

pub mod defaults;
pub mod load;
mod mixamo;
pub mod velocity;
pub mod weights;

pub use load::AvatarAnimationNodes;
use weights::{AnimationWeights, TargetAnimationWeights};

use crate::tracking::TrackingSource;

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

/// Run condition: animations only run in desktop mode.
pub(crate) fn is_desktop_mode(players: Query<&TrackingSource>) -> bool {
    players
        .iter()
        .any(|source| *source == TrackingSource::Desktop)
}

/// Marker to track which animation players have been initialized.
#[derive(Component)]
pub(crate) struct AnimationPlayerInitialized;

/// Initializes animation components when AnimationPlayer is added.
pub(crate) fn init_animation_players(
    mut commands: Commands,
    animation_players: Query<(Entity, &ChildOf), (With<AnimationPlayer>, Without<AnimationPlayerInitialized>)>,
    animation_nodes: Query<&AnimationGraphHandle, With<AvatarAnimationNodes>>,
) {
    for (entity, parent) in animation_players.iter() {
        let Ok(graph) = animation_nodes.get(parent.parent()) else {
            continue;
        };

        commands.entity(entity).insert((
            AnimationWeights::default(),
            TargetAnimationWeights::default(),
            graph.clone(),
            AnimationPlayerInitialized,
        ));
    }
}
