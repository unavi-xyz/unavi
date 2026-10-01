//! Turns a prim's script attribute into a running [`crate::Script`].

use bevy::prelude::*;
use bevy_hsd::attributes::script::HsdScript;

use crate::{
    Script,
    load::asset::Wasm,
};

/// Every peer runs every script; one that should act once decides at run time,
/// through `wired:peer/authority`.
pub fn load_hsd_scripts(
    trigger: On<Add, HsdScript>,
    scripts: Query<&HsdScript>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Ok(script) = scripts.get(trigger.entity) else {
        return;
    };
    let handle = asset_server.add(Wasm::new(script.0.clone()));
    commands.entity(trigger.entity).insert(Script(handle));
}
