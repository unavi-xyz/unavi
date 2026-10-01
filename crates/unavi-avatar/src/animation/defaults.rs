use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use unavi_assets::DEFAULT_CHARACTER_ANIMATIONS;

use super::{
    AnimationName,
    load::AvatarAnimationClips,
};

#[must_use]
pub fn default_character_animations(asset_server: &AssetServer) -> AvatarAnimationClips {
    let handle = asset_server.load(unavi_assets::path(&DEFAULT_CHARACTER_ANIMATIONS));

    let mut indices = HashMap::default();
    indices.insert(AnimationName::Falling, 0);
    indices.insert(AnimationName::Idle, 1);
    indices.insert(AnimationName::WalkLeft, 2);
    indices.insert(AnimationName::WalkRight, 3);
    indices.insert(AnimationName::Sprint, 4);
    indices.insert(AnimationName::Walk, 5);

    AvatarAnimationClips { handle, indices }
}
