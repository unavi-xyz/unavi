#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    unavi_app::App::new()
        .add_plugins(unavi_app::UnaviPlugin {
            file_path: "../assets".to_string(),
        })
        .run();
}
