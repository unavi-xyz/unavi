#![windows_subsystem = "windows"]

use bevy::prelude::*;

fn main() {
    unavi_client::asset_download::ensure_model_assets().expect("failed to download model assets");

    let mut app = App::new();
    app.add_plugins(unavi_client::UnaviPlugin);
    app.run();
}
