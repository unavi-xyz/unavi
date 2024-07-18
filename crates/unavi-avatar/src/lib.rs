use animation::AvatarAnimationClips;
use bevy::prelude::*;
use bevy_vrm::VrmPlugins;

pub mod animation;
mod fallback;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugins)
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
    pub animations: AvatarAnimationClips,
    pub fallback: FallbackAvatar,
    pub spatial: SpatialBundle,
}

#[derive(Component, Default)]
pub struct FallbackAvatar;
