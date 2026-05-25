use std::sync::{
    Arc,
    atomic::AtomicBool,
};

use bevy::prelude::*;

use crate::{
    load::asset::Wasm,
    permissions::ApiPermissions,
};

mod engine;
pub mod firewall;
pub mod load;
pub mod permissions;
mod runtime;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            engine::EnginePlugin,
            load::LoadPlugin,
            runtime::shared::SharedRuntimePlugin,
        ));
    }
}

#[derive(Component)]
#[require(ApiPermissions, Ticking, RenderTicking)]
pub struct Script(pub Handle<Wasm>);

#[derive(Component, Default)]
pub struct Ticking(pub Arc<AtomicBool>);

#[derive(Component, Default)]
pub struct RenderTicking(pub Arc<AtomicBool>);
