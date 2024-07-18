use std::time::Duration;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use mixamo::MixamoAnimationTargets;
use vrm::VrmAnimationTargets;

mod bone_chain;
mod mixamo;
mod vrm;

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
    avatars: Query<(&AvatarAnimations, &Handle<Scene>), Without<AvatarAnimationNodes>>,
    mut scenes: ResMut<Assets<Scene>>,
) {
    for (animations, scene_handle) in avatars.iter() {
        let scene = match scenes.get_mut(scene_handle) {
            Some(v) => v,
            None => continue,
        };

        // scene.world.run_system_once(
        //     |players: Query<(Entity, &AnimationPlayer)>,
        //      mut clips: ResMut<Assets<AnimationClip>>,
        //      mut commands: Commands| {
        //         for (entity, player) in players.iter() {
        //             if let Some(nodes) = create_animation_nodes(&mut clips, animations) {
        //                 commands.entity(entity).insert(nodes);
        //             }
        //         }
        //     },
        // );
    }
}

fn create_animation_nodes(
    assets: &mut Assets<AnimationClip>,
    animations: &AvatarAnimations,
) -> Option<AvatarAnimationNodes> {
    let mut graph = AnimationGraph::default();

    let idle = add_animation_clip(assets, &mut graph, &animations.idle)?;
    let walk = add_animation_clip(assets, &mut graph, &animations.walk)?;

    Some(AvatarAnimationNodes { idle, walk })
}

fn add_animation_clip(
    assets: &mut Assets<AnimationClip>,
    graph: &mut AnimationGraph,
    handle: &Handle<AnimationClip>,
) -> Option<AnimationNodeIndex> {
    let clip = assets.get_mut(handle)?;

    let mixamo_targets = MixamoAnimationTargets::default();
    let vrm_targets = VrmAnimationTargets::default();

    let curves = clip.curves_mut();

    for (name, target) in mixamo_targets.0 {
        if let Some(curve) = curves.remove(&target) {
            let vrm_target = vrm_targets.0[&name];
            curves.insert(vrm_target, curve);
        }
    }

    Some(graph.add_clip(handle.clone(), 1.0, graph.root))
}

#[derive(Component)]
pub struct Idle;

pub fn play_avatar_animations(
    mut commands: Commands,
    mut avatars: Query<
        (
            Entity,
            &AvatarAnimationNodes,
            &mut AnimationTransitions,
            &mut AnimationPlayer,
        ),
        Without<Idle>,
    >,
) {
    for (entity, animations, mut transitions, mut player) in avatars.iter_mut() {
        transitions
            .play(&mut player, animations.idle, Duration::ZERO)
            .repeat();
        commands.entity(entity).insert(Idle);
    }
}
