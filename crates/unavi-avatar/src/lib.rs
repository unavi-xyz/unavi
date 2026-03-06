use bevy::prelude::*;
use bevy_vrm::{VrmInstance, VrmPlugins};
use unavi_assets::default_avatar_path;

pub mod animation;
pub mod bones;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugins)
            .add_observer(on_avatar_added)
            .add_observer(bones::populate_avatar_bones)
            .add_systems(
                Update,
                (
                    animation::init_animation_players,
                    animation::load::load_animation_nodes,
                    animation::velocity::calc_average_velocity,
                ),
            )
            .add_systems(FixedUpdate, animation::weights::play_avatar_animations);
    }
}

#[derive(Component, Default)]
#[require(Transform, GlobalTransform, Visibility)]
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
        .map_or_else(default_avatar_path, |p| p.0.clone());
    let vrm_handle = asset_server.load(vrm_path);
    commands
        .entity(event.entity)
        .insert(VrmInstance(vrm_handle));
}

#[derive(Component, Default)]
pub struct Grounded(pub bool);
