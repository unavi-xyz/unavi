//! Runs WebAssembly components attached to prims as scripts, and hosts the
//! `wired` protocol they call.
//!
//! [`host`] implements every call, [`bindings`] lowers it onto an engine, and
//! the engine drives each script's lifecycle.

use bevy::prelude::*;
use unavi_space::membership::MembershipPlugin;

mod bindings;
mod engine;
mod error;
mod host;
mod link_intents;
mod load;
mod quota;
mod status;

pub use crate::{
    host::shared_state::{
        event_bus::{
            EventBus,
            SpatialListener,
        },
        transforms::TransformSnapshots,
    },
    load::asset::Wasm,
    quota::QuotaExempt,
    status::ScriptStatus,
};

/// Where scripts run in the frame.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ScriptSystems {
    /// Refreshes the poses scripts read. Runs in `Update`, after whatever moves
    /// the local agent, so a script sees this frame's camera.
    Snapshot,
    /// Runs each script's `update`, after the snapshot and before documents
    /// apply their changes, so a write lands the same frame.
    Tick,
}

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MembershipPlugin>() {
            app.add_plugins(MembershipPlugin);
        }

        app.configure_sets(
            Update,
            (ScriptSystems::Snapshot, ScriptSystems::Tick)
                .chain()
                .before(bevy_hsd::HsdSystems),
        )
        .add_plugins((engine::EnginePlugin, load::LoadPlugin, host::HostPlugin));
    }
}

/// A script component attached to a prim, run from the moment it is added.
#[derive(Component)]
#[require(ScriptStatus, engine::TickClocks)]
pub struct Script(pub Handle<Wasm>);
