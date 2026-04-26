use bevy::prelude::*;

use crate::load::asset::Wasm;

mod api;
mod engine;
pub mod firewall;
pub mod load;
pub mod permissions;
mod util;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((engine::EnginePlugin, load::LoadPlugin));
    }
}

#[derive(Component)]
pub struct Script(pub Handle<Wasm>);
