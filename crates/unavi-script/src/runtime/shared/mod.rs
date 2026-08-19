use std::sync::Arc;

use bevy::prelude::*;
use hsd::{
    id::{
        DocId,
        PrimId,
    },
    state::SceneState,
};
use tokio::sync::Mutex;
use unavi_policy::{
    document::ApiName,
    registry as policy_registry,
};
use unavi_quota::Quota;

use crate::{
    error::ScriptError,
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
    pub state:       Arc<std::sync::Mutex<SceneState>>,
    pub doc_id:      DocId,
    pub prim:        PrimId,
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
    ///
    /// Read per call rather than captured at instantiation. A snapshot taken
    /// when the script started would miss a document that becomes a space
    /// after its scene realized, and would hold a grant the user has since
    /// withdrawn.
    pub fn require(&self, name: ApiName) -> Result<(), ScriptError> {
        Ok(policy_registry::get(self.doc_id).policy.require(name)?)
    }

    /// Holds every document this script can write open for the duration of one
    /// tick, so a tick suspended between two host calls never has half of its
    /// writes drawn.
    ///
    /// Documents opened *during* the tick are not covered by it — they were
    /// created by a tick already in flight, and the guard closes only what it
    /// opened.
    pub async fn open_tick(&self) -> TickGuard {
        let mut held = vec![Arc::clone(&self.state)];
        held.extend(
            self.wired_scene
                .lock()
                .await
                .docs
                .iter()
                .map(|(_, doc)| Arc::clone(&doc.state)),
        );
        for state in &held {
            if let Ok(mut state) = state.lock() {
                state.open_tick();
            }
        }
        TickGuard(held)
    }
}

/// Closes the write boundaries [`Api::open_tick`] opened, including when the
/// tick trapped or was interrupted rather than returning.
pub struct TickGuard(Vec<Arc<std::sync::Mutex<SceneState>>>);

impl Drop for TickGuard {
    fn drop(&mut self) {
        for state in &self.0 {
            if let Ok(mut state) = state.lock() {
                state.close_tick();
            }
        }
    }
}

pub struct SharedRuntimePlugin;

impl Plugin for SharedRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(wired::input::bridge::bridge_press)
            .add_observer(wired::input::bridge::bridge_release)
            .add_observer(wired::input::bridge::bridge_enter)
            .add_observer(wired::input::bridge::bridge_leave)
            .add_observer(wired::input::bridge::bridge_scroll)
            .add_observer(registry::agent::register_peers)
            .add_observer(registry::agent::register_local_agent)
            .add_observer(registry::agent::deregister_agents)
            .add_observer(registry::transform::register_nodes)
            .add_observer(registry::transform::deregister_transforms)
            .add_observer(registry::transform::deregister_doc_root)
            .add_systems(
                Update,
                (
                    registry::agent::spawn_proxy_nodes,
                    registry::pointer::snapshot_pointers,
                    wired::input::bridge::bridge_global_presses,
                    wired::input::bridge::bridge_global_scroll
                        .run_if(unavi_input::capture::scene_has_input),
                    wired::input::bridge::bridge_menu,
                ),
            )
            .add_systems(
                Update,
                (
                    (
                        bevy::transform::systems::mark_dirty_trees,
                        bevy::transform::systems::propagate_parent_transforms,
                        bevy::transform::systems::sync_simple_transforms,
                    )
                        .chain(),
                    registry::transform::snapshot_transforms,
                    registry::transform::snapshot_doc_roots,
                )
                    .chain()
                    .in_set(crate::ScriptSnapshotSet),
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
