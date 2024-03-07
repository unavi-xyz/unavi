#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{asset::AssetMetaCheck, prelude::*, utils::HashSet};

use unavi_app::UnaviPlugin;

fn main() {
    let mut meta_paths = HashSet::new();
    meta_paths.insert("images/dev-white.png".into());
    meta_paths.insert("images/skybox-1-512.png".into());

    App::new()
        .insert_resource(AssetMetaCheck::Paths(meta_paths))
        .add_plugins((DefaultPlugins, UnaviPlugin::default()))
        .run();
}
