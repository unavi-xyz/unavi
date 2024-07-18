use std::time::Duration;

use bevy::prelude::*;
use bevy_vrm::animations::vrm::VRM_ANIMATION_TARGETS;
use mixamo::MixamoAnimationTargets;

mod mixamo;

#[derive(Component, Clone)]
pub struct AvatarAnimationClips {
    pub idle: Handle<AnimationClip>,
    pub walk: Handle<AnimationClip>,
}

#[derive(Component, Clone)]
pub struct AvatarAnimationNodes {
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
}

#[derive(Component)]
pub struct CreatedAnimationGraph;

pub fn create_animation_graph(
    avatars: Query<(Entity, &AvatarAnimationClips), Without<Handle<AnimationGraph>>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (entity, animations) in avatars.iter() {
        let (graph, nodes) = match create_animation_nodes(&mut clips, animations) {
            Some(n) => n,
            None => continue,
        };

        let graph = graphs.add(graph);

        commands.entity(entity).insert((graph, nodes));
    }
}

fn create_animation_nodes(
    clips: &mut Assets<AnimationClip>,
    animations: &AvatarAnimationClips,
) -> Option<(AnimationGraph, AvatarAnimationNodes)> {
    let mut graph = AnimationGraph::default();

    let idle = add_animation_clip(clips, &mut graph, &animations.idle)?;
    let walk = add_animation_clip(clips, &mut graph, &animations.walk)?;

    Some((graph, AvatarAnimationNodes { idle, walk }))
}

fn add_animation_clip(
    clips: &mut Assets<AnimationClip>,
    graph: &mut AnimationGraph,
    handle: &Handle<AnimationClip>,
) -> Option<AnimationNodeIndex> {
    let clip = clips.get_mut(handle)?;

    let mixamo_targets = MixamoAnimationTargets::default();

    let curves = clip.curves_mut();

    for (name, target) in mixamo_targets.0 {
        if let Some(curve) = curves.remove(&target) {
            let vrm_target = VRM_ANIMATION_TARGETS[&name];
            curves.insert(vrm_target, curve);
        }
    }

    Some(graph.add_clip(handle.clone(), 1.0, graph.root))
}

#[derive(Component)]
pub struct Idle;

pub fn play_avatar_animations(
    avatars: Query<(&Handle<AnimationGraph>, &AvatarAnimationNodes)>,
    mut commands: Commands,
    mut new_players: Query<(Entity, &mut AnimationPlayer, &Parent), Added<AnimationPlayer>>,
) {
    for (entity, mut player, parent) in new_players.iter_mut() {
        if let Ok((graph, nodes)) = avatars.get(parent.get()) {
            let mut transitions = AnimationTransitions::default();

            transitions
                .play(&mut player, nodes.idle, Duration::ZERO)
                .repeat();

            commands
                .entity(entity)
                .insert((graph.clone(), nodes.clone(), transitions));
        }
    }
}
