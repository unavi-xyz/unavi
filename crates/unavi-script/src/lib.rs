use std::sync::{
    Arc,
    atomic::{
        AtomicBool,
        Ordering,
    },
};

use bevy::prelude::*;
use unavi_policy::PolicyPlugin;

use crate::load::asset::Wasm;

#[cfg(feature = "debug")] pub mod debug;
mod engine;
pub mod error;
pub mod load;
mod portal_host;
pub mod quota;
pub mod runtime;

/// Refreshes the transform snapshot to the current frame's poses.
///
/// Runs in `Update` after agent movement so per-frame script reads observe this
/// frame's camera rather than the previous frame's. Script execution runs after
/// this set.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ScriptSnapshotSet;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PolicyPlugin>() {
            app.add_plugins(PolicyPlugin);
        }

        app.add_plugins((
            engine::EnginePlugin,
            load::LoadPlugin,
            runtime::shared::SharedRuntimePlugin,
        ))
        .add_systems(
            FixedUpdate,
            (
                portal_host::service_portal_watches,
                portal_host::drain_pending,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
#[require(FixedUpdating, Trapped, Updating)]
pub struct Script(pub Handle<Wasm>);

#[derive(Component, Default)]
pub struct FixedUpdating(pub Arc<AtomicBool>);

#[derive(Component, Default)]
pub struct Updating(pub Arc<AtomicBool>);

/// Whether this script's instance has trapped, and is therefore finished.
///
/// A trap leaves a component instance permanently un-enterable — every call
/// after it fails with "cannot enter component instance" — so a trapped script
/// is driven no further. Shared rather than a plain flag because the tick that
/// discovers the trap runs off the world.
#[derive(Component, Default)]
pub struct Trapped(pub Arc<AtomicBool>);

impl Trapped {
    /// Records the trap, answering whether this was the one that found it —
    /// so the failure is reported once rather than every frame forever.
    #[must_use]
    pub fn set(&self) -> bool {
        !self.0.swap(true, Ordering::SeqCst)
    }

    #[must_use]
    pub fn get(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
