#![windows_subsystem = "windows"]

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(unavi_client::UnaviPlugin);
    app.run();
}
