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
                (
                    fallback::despawn_fallback_children,
                    fallback::remove_fallback_avatar,
                    fallback::spawn_fallback_children,
                ),
            );
    }
}

#[derive(Component, Default)]
pub struct FallbackAvatar;
