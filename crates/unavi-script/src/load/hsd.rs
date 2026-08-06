use bevy::prelude::*;
use bevy_hsd::attributes::script::HsdScript;

use crate::{
    Script,
    load::asset::Wasm,
};

/// Every peer runs every script; a script differentiates owner-only logic at
/// runtime via `is_self_owner`, rather than being skipped on non-owners.
pub fn load_hsd_scripts(
    trigger: On<Add, HsdScript>,
    scripts: Query<&HsdScript>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let script = scripts.get(trigger.entity).expect("get scripts");
    let handle = asset_server.add(Wasm(script.0.clone()));
    commands.entity(trigger.entity).insert(Script(handle));
}
