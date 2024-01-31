#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

use unavi_app::UnaviPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UnaviPlugin::default()))
        .run();
}

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
fn start() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            UnaviPlugin::default(),
        ))
        .run();
}
