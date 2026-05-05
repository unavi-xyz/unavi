use std::sync::Arc;

use bevy::prelude::*;
use tokio::sync::Mutex;

use crate::runtime::shared::wired::{input::WiredInputBackend, scene::WiredSceneBackend};

pub mod registry;
mod slot_map;
pub mod wired;

#[derive(Clone)]
pub struct RuntimeBackend {
    pub wired_input: Arc<Mutex<WiredInputBackend>>,
    pub wired_scene: Arc<Mutex<WiredSceneBackend>>,
}

pub struct SharedRuntimePlugin;

impl Plugin for SharedRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(
            wired::input::bridge::bridge_squeeze_down.pipe(wired::input::bridge::send_to_listeners),
        )
        .add_observer(
            wired::input::bridge::bridge_squeeze_up.pipe(wired::input::bridge::send_to_listeners),
        )
        .add_observer(registry::transform::register_nodes)
        .add_observer(registry::transform::deregister_transforms)
        .add_systems(
            Update,
            (
                registry::transform::snapshot_transforms,
                wired::input::bridge::bridge_menu_desktop
                    .pipe(wired::input::bridge::send_to_listeners),
                wired::input::bridge::bridge_menu_left
                    .pipe(wired::input::bridge::send_to_listeners),
                wired::input::bridge::bridge_menu_right
                    .pipe(wired::input::bridge::send_to_listeners),
            ),
        );
    }
}
