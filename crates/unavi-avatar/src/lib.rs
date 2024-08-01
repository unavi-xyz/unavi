use animation::AvatarAnimationClips;
use bevy::prelude::*;
use bevy_vrm::VrmPlugins;

pub mod animation;
mod defaults;
mod fallback;
mod velocity;

pub use defaults::*;
pub use fallback::FallbackAvatar;
pub use velocity::AverageVelocity;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugins)
            .add_systems(Startup, fallback::init_fallback_assets)
            .add_systems(
                Update,
                (
                    animation::init_animations,
                    animation::load::load_animation_nodes,
                    animation::play_avatar_animations,
                    fallback::despawn_fallback_children,
                    fallback::remove_fallback_avatar,
                    fallback::spawn_fallback_children,
                    velocity::calc_average_velocity,
                ),
            );
    }
}

#[derive(Bundle)]
pub struct AvatarBundle {
    pub animations: AvatarAnimationClips,
    pub fallback: FallbackAvatar,
    pub velocity: AverageVelocity,
}

impl AvatarBundle {
    pub fn new(animations: AvatarAnimationClips) -> Self {
        Self {
            animations,
            fallback: FallbackAvatar,
            velocity: AverageVelocity::default(),
        }
    }
}
