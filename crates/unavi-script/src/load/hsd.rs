use bevy::prelude::*;
use bevy_hsd::{
    HsdChild,
    HsdRecordId,
    attributes::script::HsdScript,
};
use bevy_wds::blob::{
    deps::BlobDep,
    request::{
        BlobRequest,
        BlobResponse,
    },
};
use unavi_space::{
    membership::doc_space,
    peer::self_peer_id,
    state::owner::doc_owner,
};

use crate::{
    Script,
    load::asset::Wasm,
};

#[derive(Component)]
pub struct PendingScript;

pub fn load_hsd_scripts(
    trigger: On<Add, HsdScript>,
    scripts: Query<&HsdScript>,
    children: Query<&HsdChild>,
    records: Query<&HsdRecordId>,
    mut commands: Commands,
) {
    // A doc owned by another peer is a pure replica synced over WDS; running its
    // script locally would duplicate its spawns. Only run ours or unowned docs.
    if let Ok(doc) = children.get(trigger.entity).and_then(|c| records.get(c.0))
        && doc_space(doc.0)
            .and_then(|space| doc_owner(space, doc.0))
            .is_some_and(|owner| Some(owner) != self_peer_id())
    {
        return;
    }

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
