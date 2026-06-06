use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdRecordId,
};
use blake3::Hash;
use unavi_space::{
    Space,
    membership::SpaceOwner,
    peer::Peer,
};

use crate::quota::registry::{
    forget_peer,
    forget_space,
    reassign_document_in_space,
};

/// Repoints a document's quota at its owner when it joins or changes space,
/// migrating standing usage off the previous owner.
pub fn reassign_doc_quota(
    trigger: On<Insert, SpaceOwner>,
    docs: Query<(&HsdRecordId, &SpaceOwner), With<Hsd>>,
    spaces: Query<&Space>,
) {
    let Ok((record, owner)) = docs.get(trigger.entity) else {
        return;
    };
    let Ok(space) = spaces.get(owner.0) else {
        return;
    };
    reassign_document_in_space(record.0, space.0);
}

/// Sheds a departed space's quota so the table does not retain dead scopes.
pub fn forget_space_quota(trigger: On<Remove, Space>, spaces: Query<&Space>) {
    if let Ok(space) = spaces.get(trigger.entity) {
        forget_space(space.0);
    }
}

/// Sheds a disconnected peer's quota the same way [`forget_space_quota`] does.
pub fn forget_peer_quota(trigger: On<Remove, Peer>, peers: Query<&Peer>) {
    if let Ok(peer) = peers.get(trigger.entity) {
        forget_peer(Hash::from(*peer.0.id.as_bytes()));
    }
}
