use bevy::prelude::*;

use crate::{load::asset::Wasm, permissions::ApiPermissions};

mod engine;
pub mod firewall;
pub mod load;
pub mod permissions;
mod runtime;
mod util;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((engine::EnginePlugin, load::LoadPlugin));
    }
}

#[derive(Component)]
#[require(ApiPermissions)]
pub struct Script(pub Handle<Wasm>);
