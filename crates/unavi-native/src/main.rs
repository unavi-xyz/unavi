#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(unavi_app::UnaviPlugin {
            file_path: "../assets".to_string(),
            ..default()
        })
        .run();
}
