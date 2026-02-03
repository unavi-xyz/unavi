//! VRM avatar visual representation and animation.
//!
//! This crate provides core avatar types, spawning, and animation systems
//! for VRM models.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_vrm::{BoneName, VrmInstance, VrmPlugins};
use unavi_assets::default_avatar_path;

pub mod animation;

pub use animation::{
    AnimationName, AnimationPlayerInitialized, AvatarAnimationNodes, bone_mask_group,
    defaults::default_character_animations,
    load::AvatarAnimationClips,
    velocity::AverageVelocity,
    weights::{AnimationWeights, TargetAnimationWeights},
};

/// Avatar plugin for visual representation.
pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugins)
            .add_systems(
                Update,
                (
                    animation::init_animation_players,
                    animation::load::load_animation_nodes,
                    animation::velocity::calc_average_velocity,
                    populate_avatar_bones,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    animation::weights::init_animation_weights,
                    animation::weights::play_avatar_animations,
                ),
            );
    }
}

/// Marker for an avatar entity.
#[derive(Component, Default)]
#[require(Transform, GlobalTransform, Visibility)]
pub struct Avatar;

/// Whether the entity is grounded (not airborne).
#[derive(Component, Default)]
pub struct Grounded(pub bool);

/// Builder for spawning a visual avatar entity.
#[derive(Default)]
pub struct AvatarSpawner {
    pub vrm_asset: Option<String>,
}

impl AvatarSpawner {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_vrm(mut self, vrm_asset: impl Into<String>) -> Self {
        self.vrm_asset = Some(vrm_asset.into());
        self
    }

    /// Spawns a visual avatar entity.
    /// Returns the avatar entity ID.
    pub fn spawn(&self, commands: &mut Commands, asset_server: &AssetServer) -> Entity {
        let vrm_path = self.vrm_asset.clone().unwrap_or_else(default_avatar_path);
        let vrm_handle = asset_server.load(vrm_path);

        commands
            .spawn((
                Avatar,
                AvatarBones::default(),
                VrmInstance(vrm_handle),
                Transform::default(),
            ))
            .id()
    }
}

/// Maps tracked poses to VRM avatar bones.
#[derive(Component, Default, Deref, DerefMut)]
pub struct AvatarBones(pub HashMap<BoneName, Entity>);

/// Marker to track which avatars have had bones populated.
#[derive(Component)]
pub struct AvatarBonesPopulated;

/// Finds and stores VRM bone entities for avatars with loaded scenes.
pub fn populate_avatar_bones(
    mut commands: Commands,
    avatars: Query<(Entity, &AvatarBones), (With<VrmInstance>, Without<AvatarBonesPopulated>)>,
    bones: Query<(Entity, &BoneName)>,
    parents: Query<&ChildOf>,
) {
    for (avatar_ent, _) in avatars.iter() {
        let mut avatar_bones = HashMap::new();

        for (bone_ent, bone_name) in bones.iter() {
            if !is_child(bone_ent, avatar_ent, &parents) {
                continue;
            }

            avatar_bones.insert(*bone_name, bone_ent);
        }

        if avatar_bones.contains_key(&BoneName::Head) {
            commands
                .entity(avatar_ent)
                .insert((AvatarBones(avatar_bones), AvatarBonesPopulated));
        }
    }
}

/// Walks up the parent tree, searching for a specific Entity.
fn is_child(target_child: Entity, target_parent: Entity, parents: &Query<&ChildOf>) -> bool {
    if target_child == target_parent {
        true
    } else if let Ok(child_of) = parents.get(target_child) {
        is_child(child_of.parent(), target_parent, parents)
    } else {
        false
    }
}
