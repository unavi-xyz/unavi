use bevy::{platform::collections::HashMap, prelude::*};
use bevy_gltf_kun::import::gltf::{animation::RawGltfAnimation, loader::GltfLoaderSettings};
use constcat::concat;

use crate::animation::AnimationName;

use super::load::{AvatarAnimation, AvatarAnimationClips};

const DEFAULT_ANIMATIONS: &str = concat!(unavi_constants::CDN_ASSETS_URL, "/models/animations.glb");

pub fn default_character_animations(asset_server: &AssetServer) -> AvatarAnimationClips {
    let mut map = HashMap::default();

    let gltf = asset_server.load(DEFAULT_ANIMATIONS);

    let load_animation = |i: usize| -> AvatarAnimation {
        let animation = asset_server.load_with_settings::<RawGltfAnimation, GltfLoaderSettings>(
            format!("{DEFAULT_ANIMATIONS}#RawAnimation{i}"),
            |settings| {
                settings.expose_raw_animation_curves = true;
            },
        );
        AvatarAnimation {
            gltf: gltf.clone(),
            animation,
        }
    };

    map.insert(AnimationName::Falling, load_animation(0));
    map.insert(AnimationName::Idle, load_animation(1));
    map.insert(AnimationName::WalkLeft, load_animation(2));
    map.insert(AnimationName::WalkRight, load_animation(3));
    map.insert(AnimationName::Sprint, load_animation(4));
    map.insert(AnimationName::Walk, load_animation(5));

    AvatarAnimationClips(map)
}
