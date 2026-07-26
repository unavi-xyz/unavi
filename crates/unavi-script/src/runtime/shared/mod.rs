use std::sync::Arc;

use bevy::prelude::*;
use blake3::Hash;
use loro::{
    LoroDoc,
    TreeID,
};
use tokio::sync::Mutex;
use unavi_quota::Quota;

use crate::{
    error::ScriptError,
    permissions::{
        ApiName,
        ApiPermissions,
    },
    runtime::shared::wired::{
        agent::WiredAgentApi,
        event::WiredEventApi,
        input::WiredInputApi,
        kv::WiredKvApi,
        scene::WiredSceneApi,
        wds::WiredWdsApi,
    },
};

pub mod registry;
mod slot_map;
pub mod wired;

pub struct Api {
    pub doc:         Arc<LoroDoc>,
    pub doc_id:      Hash,
    pub prim:        TreeID,
    pub permissions: ApiPermissions,
    pub quota:       Arc<Quota>,
    pub wired_agent: Mutex<WiredAgentApi>,
    pub wired_event: Mutex<WiredEventApi>,
    pub wired_input: Mutex<WiredInputApi>,
    pub wired_kv:    Mutex<WiredKvApi>,
    pub wired_scene: Mutex<WiredSceneApi>,
    pub wired_wds:   Mutex<WiredWdsApi>,
}

impl Api {
    /// Gates a call on the document holding the named permission.
    pub fn require(&self, name: ApiName) -> Result<(), ScriptError> {
        if self.permissions.contains(&name) {
            Ok(())
        } else {
            Err(ScriptError::permission(format!("{name:?}")))
        }
    }
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
        .add_observer(registry::agent::register_peers)
        .add_observer(registry::agent::register_local_agent)
        .add_observer(registry::agent::deregister_agents)
        .add_observer(registry::firewall::register_docs)
        .add_observer(registry::firewall::register_subdoc_firewall)
        .add_observer(registry::firewall::deregister_firewalls)
        .add_observer(registry::quota::reassign_doc_quota)
        .add_observer(registry::quota::forget_space_quota)
        .add_observer(registry::quota::forget_peer_quota)
        .add_observer(registry::transform::register_nodes)
        .add_observer(registry::transform::deregister_transforms)
        .add_observer(registry::transform::deregister_doc_root)
        .add_systems(
            Update,
            (
                registry::agent::spawn_proxy_nodes,
                wired::input::bridge::bridge_menu_desktop
                    .pipe(wired::input::bridge::send_to_listeners),
                wired::input::bridge::bridge_menu_left
                    .pipe(wired::input::bridge::send_to_listeners),
                wired::input::bridge::bridge_menu_right
                    .pipe(wired::input::bridge::send_to_listeners),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                registry::transform::snapshot_transforms,
                registry::transform::snapshot_doc_roots,
            )
                .after(bevy::transform::TransformSystems::Propagate),
        );
    }
}
