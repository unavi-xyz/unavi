use bevy::prelude::*;
use mixamo::MixamoAnimationTargets;

mod mixamo;

#[derive(Component)]
pub struct AvatarAnimations {
    pub idle: Handle<AnimationClip>,
    pub walk: Handle<AnimationClip>,
}

#[derive(Component)]
pub struct AvatarAnimationNodes {
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
}

pub fn create_animation_graph(
    assets: Res<Assets<AnimationClip>>,
    avatars: Query<(Entity, &AvatarAnimations), Without<AvatarAnimationNodes>>,
    commands: Commands,
) {
    for (entity, animations) in avatars.iter() {
        let idle = match assets.get(&animations.idle) {
            Some(v) => v,
            None => continue,
        };

        let mixamo_targets = MixamoAnimationTargets::default();

        // let mut graph = AnimationGraph::default();
        // let idle = graph.add_clip(idle, 1.0, graph.root);
        // let walk = graph.add_clip(walk, 1.0, graph.root);
        //
        // commands
        //     .entity(entity)
        //     .insert(AvatarAnimationNodes { idle, walk });
    }
}

pub fn apply_avatar_animations(mut avatars: Query<(&AvatarAnimationNodes, &mut AnimationPlayer)>) {
    for (animations, mut player) in avatars.iter_mut() {
        player.play(animations.idle);
    }
}
