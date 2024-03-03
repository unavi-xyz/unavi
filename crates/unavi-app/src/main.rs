#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

use unavi_app::UnaviPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UnaviPlugin::default()))
        .run();
}
