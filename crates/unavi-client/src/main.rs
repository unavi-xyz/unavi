#![windows_subsystem = "windows"]

use bevy::prelude::*;

fn main() {
    App::new().add_plugins(unavi_client::UnaviPlugin).run();
}
