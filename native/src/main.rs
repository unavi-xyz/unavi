#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    unavi_app::start(unavi_app::StartOptions {
        asset_folder: "../assets".to_string(),
        ..Default::default()
    });
}
