use animation::AvatarAnimations;
use bevy::prelude::*;
use bevy_vrm::VrmPlugin;

pub mod animation;
mod fallback;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugin)
            .register_type::<bevy_vrm::SpringBones>()
            .add_systems(Startup, fallback::init_fallback_assets)
            .add_systems(
                Update,
                (
                    animation::play_avatar_animations,
                    animation::create_animation_graph,
                    fallback::despawn_fallback_children,
                    fallback::remove_fallback_avatar,
                    fallback::spawn_fallback_children,
                ),
            );
    }
}

#[derive(Bundle)]
pub struct AvatarBundle {
    pub animation_player: AnimationPlayer,
    pub animations: AvatarAnimations,
    pub fallback: FallbackAvatar,
    pub spatial: SpatialBundle,
    pub transitions: AnimationTransitions,
}

#[derive(Component, Default)]
pub struct FallbackAvatar;
