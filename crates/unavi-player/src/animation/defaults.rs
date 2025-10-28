use bevy::{platform::collections::HashMap, prelude::*};
use bevy_gltf_kun::import::gltf::{animation::RawGltfAnimation, loader::GltfLoaderSettings};

use crate::animation::AnimationName;

use super::load::{AvatarAnimation, AvatarAnimationClips};

const ASSET: &str = "models/animations.glb";

pub fn default_character_animations(asset_server: &AssetServer) -> AvatarAnimationClips {
    let mut map = HashMap::default();

    let load_animation = |i: usize| -> AvatarAnimation {
        let handle = asset_server.load_with_settings::<RawGltfAnimation, GltfLoaderSettings>(
            format!("{ASSET}#RawAnimation{i}"),
            |settings| {
                settings.expose_raw_animation_curves = true;
            },
        );
        AvatarAnimation(handle)
    };

    map.insert(AnimationName::Falling, load_animation(0));
    map.insert(AnimationName::Idle, load_animation(1));
    map.insert(AnimationName::WalkLeft, load_animation(2));
    map.insert(AnimationName::WalkRight, load_animation(3));
    map.insert(AnimationName::Walk, load_animation(5));

    AvatarAnimationClips(map)
}
