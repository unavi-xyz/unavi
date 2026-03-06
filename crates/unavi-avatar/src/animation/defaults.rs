use bevy::{platform::collections::HashMap, prelude::*};
use bevy_gltf_kun::import::gltf::{animation::RawGltfAnimation, loader::GltfLoaderSettings};
use unavi_assets::{default_character_animations_path, default_menu_animation_path};

use super::{
    AnimationName,
    load::{AvatarAnimation, AvatarAnimationClips},
};

#[must_use]
pub fn default_character_animations(asset_server: &AssetServer) -> AvatarAnimationClips {
    let mut map = HashMap::default();

    let path_char = default_character_animations_path();
    let path_menu = default_menu_animation_path();

    let load_animation = |path: &str, i: usize| -> AvatarAnimation {
        let animation = asset_server.load_with_settings::<RawGltfAnimation, GltfLoaderSettings>(
            format!("{path}#RawAnimation{i}"),
            |settings| {
                settings.expose_raw_animation_curves = true;
            },
        );
        let gltf = asset_server.load(path.to_string());
        AvatarAnimation { gltf, animation }
    };

    map.insert(AnimationName::Falling, load_animation(&path_char, 0));
    map.insert(AnimationName::Idle, load_animation(&path_char, 1));
    map.insert(AnimationName::WalkLeft, load_animation(&path_char, 2));
    map.insert(AnimationName::WalkRight, load_animation(&path_char, 3));
    map.insert(AnimationName::Sprint, load_animation(&path_char, 4));
    map.insert(AnimationName::Walk, load_animation(&path_char, 5));
    map.insert(AnimationName::Menu, load_animation(&path_menu, 0));

    AvatarAnimationClips(map)
}
