#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    unavi_app::start(unavi_app::StartOptions {
        file_path: "../assets".to_string(),
        ..Default::default()
    });
}
