use std::collections::HashMap;

use bevy::{
    prelude::*,
    world_serialization::WorldInstanceReady,
};
use bevy_vrm::instance::VrmInstanceReady;
use serde_vrm::vrm1::vrmc_vrm::humanoid::HumanBoneName;

#[derive(Component, Deref, DerefMut)]
pub struct AvatarBones(pub HashMap<HumanBoneName, Entity>);

/// Collects an avatar's bones once its VRM scene has finished spawning.
/// `WorldInstanceReady` fires once the whole hierarchy under the avatar
/// entity exists, so a single descendant walk is enough; no need to re-scan
/// every tick waiting for bones to appear.
pub(crate) fn populate_avatar_bones(
    ready: On<WorldInstanceReady>,
    avatars: Query<(), (With<VrmInstanceReady>, Without<AvatarBones>)>,
    children: Query<&Children>,
    bone_names: Query<&HumanBoneName>,
    mut commands: Commands,
) {
    let entity = ready.entity;
    if !avatars.contains(entity) {
        return;
    }

    let avatar_bones: HashMap<HumanBoneName, Entity> = children
        .iter_descendants(entity)
        .filter_map(|descendant| {
            bone_names
                .get(descendant)
                .ok()
                .map(|name| (*name, descendant))
        })
        .collect();

    if avatar_bones.is_empty() {
        return;
    }

    commands.entity(entity).insert(AvatarBones(avatar_bones));
}
