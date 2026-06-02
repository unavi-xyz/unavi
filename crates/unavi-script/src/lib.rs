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
    },
};

#[cfg(feature = "debug")] pub mod debug;
mod engine;
pub mod firewall;
pub mod load;
pub mod permissions;
mod portal_host;
pub mod runtime;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            engine::EnginePlugin,
            load::LoadPlugin,
            runtime::shared::SharedRuntimePlugin,
        ))
        .add_observer(grant_space_permissions)
        .add_observer(portal_host::enqueue_incoming_on_destination)
        .add_observer(portal_host::enqueue_incoming_on_doc_load)
        .add_observer(portal_host::enqueue_backlink_on_accept)
        .add_systems(Update, portal_host::drain_pending);
    }
}

#[derive(Component)]
#[require(ApiPermissions, Ticking, RenderTicking)]
pub struct Script(pub Handle<Wasm>);

#[derive(Component, Default)]
pub struct Ticking(pub Arc<AtomicBool>);

#[derive(Component, Default)]
pub struct RenderTicking(pub Arc<AtomicBool>);
