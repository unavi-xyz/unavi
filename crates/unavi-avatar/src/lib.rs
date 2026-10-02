//! VRM avatars: loading, Mixamo-retargeted locomotion animation, and the
//! bone map scripts and the agent read the rig from. Owns no input and no
//! physics; `unavi-agent` drives an avatar's rig and feeds it
//! [`animation::locomotion::LocomotionProfile`].

use bevy::prelude::*;
use bevy_vrm::{
    VrmInstance,
    VrmPlugins,
};
use unavi_assets::DEFAULT_AVATAR;

pub mod animation;
pub mod bones;

/// Adds VRM loading, retargeted locomotion animation, and bone tracking.
/// Avatars animate from their own [`Transform`] and
/// [`animation::velocity::AverageVelocity`] target; nothing here reads
/// input.
pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugins)
            .init_asset::<animation::raw::RawAnimations>()
            .init_asset_loader::<animation::raw::RawAnimationsLoader>()
            .add_observer(on_avatar_added)
            .add_observer(bones::populate_avatar_bones)
            .add_systems(
                Update,
                (
                    animation::init_animation_players,
                    animation::load::load_animation_nodes,
                    animation::velocity::calc_average_velocity,
                    animation::weights::play_avatar_animations,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Default)]
#[require(Transform, Visibility, animation::locomotion::LocomotionProfile)]
pub struct Avatar;

#[derive(Component, Clone, Deref)]
pub struct VrmPath(pub String);

fn on_avatar_added(
    event: On<Add, Avatar>,
    vrm_paths: Query<&VrmPath>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    let vrm_path = vrm_paths
        .get(event.entity)
        .ok()
        .map_or_else(|| unavi_assets::path(&DEFAULT_AVATAR), |p| p.0.clone());
    let vrm_handle = asset_server.load(vrm_path);
    commands
        .entity(event.entity)
        .insert(VrmInstance(vrm_handle));
}

#[derive(Component, Default)]
pub struct Grounded(pub bool);
