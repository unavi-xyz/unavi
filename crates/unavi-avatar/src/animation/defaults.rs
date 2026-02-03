use bevy::{platform::collections::HashMap, prelude::*};
use bevy_gltf_kun::import::gltf::{animation::RawGltfAnimation, loader::GltfLoaderSettings};
use unavi_assets::default_animations_path;

use super::{
    AnimationName,
    load::{AvatarAnimation, AvatarAnimationClips},
};

#[must_use]
pub fn default_character_animations(asset_server: &AssetServer) -> AvatarAnimationClips {
    let mut map = HashMap::default();

    let animations_path = default_animations_path();
    let gltf = asset_server.load(&animations_path);

    let load_animation = |i: usize| -> AvatarAnimation {
        let animation = asset_server.load_with_settings::<RawGltfAnimation, GltfLoaderSettings>(
            format!("{animations_path}#RawAnimation{i}"),
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
