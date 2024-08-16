use bevy::{prelude::*, utils::HashMap};

use crate::animation::{AnimationName, AvatarAnimation, AvatarAnimationClips};

pub const DEFAULT_VRM: &str = "models/robot.vrm";

pub fn default_character_animations(asset_server: &AssetServer) -> AvatarAnimationClips {
    let mut map = HashMap::default();

    map.insert(
        AnimationName::Falling,
        AvatarAnimation {
            clip: asset_server.load("models/character-animations.glb#Animation0"),
            gltf: asset_server.load("models/character-animations.glb"),
        },
    );
    map.insert(
        AnimationName::Idle,
        AvatarAnimation {
            clip: asset_server.load("models/character-animations.glb#Animation1"),
            gltf: asset_server.load("models/character-animations.glb"),
        },
    );
    map.insert(
        AnimationName::Menu,
        AvatarAnimation {
            clip: asset_server.load("models/idle-menu.glb#Animation0"),
            gltf: asset_server.load("models/idle-menu.glb"),
        },
    );
    map.insert(
        AnimationName::WalkLeft,
        AvatarAnimation {
            clip: asset_server.load("models/character-animations.glb#Animation2"),
            gltf: asset_server.load("models/character-animations.glb"),
        },
    );
    map.insert(
        AnimationName::WalkRight,
        AvatarAnimation {
            clip: asset_server.load("models/character-animations.glb#Animation3"),
            gltf: asset_server.load("models/character-animations.glb"),
        },
    );
    map.insert(
        AnimationName::Walk,
        AvatarAnimation {
            clip: asset_server.load("models/character-animations.glb#Animation5"),
            gltf: asset_server.load("models/character-animations.glb"),
        },
    );

    AvatarAnimationClips(map)
}
