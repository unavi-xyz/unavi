use std::sync::{
    Arc,
    atomic::AtomicBool,
};

use bevy::prelude::*;

use crate::{
    load::asset::Wasm,
    permissions::{
        ApiPermissions,
        grant_space_permissions,
        inherit_host_permissions,
    },
};

#[cfg(feature = "debug")] pub mod debug;
mod engine;
pub mod error;
pub mod firewall;
pub mod load;
pub mod permissions;
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
        app.add_plugins((
            engine::EnginePlugin,
            load::LoadPlugin,
            runtime::shared::SharedRuntimePlugin,
        ))
        .add_observer(grant_space_permissions)
        .add_observer(inherit_host_permissions)
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
#[require(ApiPermissions, FixedUpdating, Updating)]
pub struct Script(pub Handle<Wasm>);

#[derive(Component, Default)]
pub struct FixedUpdating(pub Arc<AtomicBool>);

#[derive(Component, Default)]
pub struct Updating(pub Arc<AtomicBool>);
