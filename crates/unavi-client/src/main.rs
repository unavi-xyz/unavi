#![windows_subsystem = "windows"]

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(unavi_client::UnaviPlugin);

    unavi_client::asset_download::copy_assets_to_dirs().expect("failed to copy assets");
    unavi_client::asset_download::ensure_downloaded_assets()
        .expect("failed to download model assets");

    app.run();
}
