use bevy::prelude::*;
use blake3::Hash;
use loro::TreeID;
use tokio::sync::Mutex;

use crate::{
    permissions::ApiPermissions,
    runtime::shared::wired::{input::WiredInputApi, scene::WiredSceneApi},
};

pub mod registry;
mod slot_map;
pub mod wired;

pub struct Api {
    pub document: Hash,
    pub node: TreeID,
    pub permissions: ApiPermissions,
    pub wired_input: Mutex<WiredInputApi>,
    pub wired_scene: Mutex<WiredSceneApi>,
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
        .add_observer(registry::firewall::register_docs)
        .add_observer(registry::firewall::deregister_firewalls)
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
