use bevy::prelude::*;
use bevy_hsd::attributes::script::HsdScript;
use bevy_wds::blob::{
    deps::BlobDep,
    request::{
        BlobRequest,
        BlobResponse,
    },
};

use crate::{
    Script,
    load::asset::Wasm,
};

#[derive(Component)]
pub struct PendingScript;

/// Every peer runs every script; a script differentiates owner-only logic at
/// runtime via `is_self_owner`, rather than being skipped on non-owners.
pub fn load_hsd_scripts(
    trigger: On<Add, HsdScript>,
    scripts: Query<&HsdScript>,
    mut commands: Commands,
) {
    let script = scripts.get(trigger.entity).expect("get scripts");
    commands.spawn((
        BlobDep(trigger.entity),
        BlobRequest(script.0),
        PendingScript,
    ));
}

pub fn process_pending_scripts(
    pending: Query<(Entity, &BlobDep, &mut BlobResponse), With<PendingScript>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (entity, parent, mut res) in pending {
        commands.entity(entity).despawn();

        let Some(bytes) = res.0.take() else {
            continue;
        };
        let handle = asset_server.add(Wasm(bytes.to_vec()));

        commands.entity(parent.0).insert(Script(handle));
    }
}
