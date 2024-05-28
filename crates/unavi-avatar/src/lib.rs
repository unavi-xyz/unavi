use bevy::prelude::*;
use bevy_vrm::VrmPlugin;

mod fallback;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugin)
            .add_systems(Startup, fallback::init_fallback_assets)
            .add_systems(
                Update,
                (fallback::spawn_fallbacks, fallback::despawn_fallbacks),
            );
    }
}

#[derive(Bundle, Default)]
pub struct AvatarBundle {
    avatar: Avatar,
    fallback: FallbackAvatar,
}

#[derive(Component, Default)]
pub struct Avatar;

#[derive(Component, Default)]
pub struct FallbackAvatar;
