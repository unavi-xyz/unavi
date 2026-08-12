use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdDocId,
};
use iroh_docs::NamespaceId;
use unavi_quota::registry::{
    forget_peer,
    forget_space,
};
use unavi_space::{
    Space,
    membership::SpaceOwner,
    peer::Peer,
    quota::reassign_document_in_space,
};

/// Repoints a document's quota at its owner when it joins or changes space,
/// migrating standing usage off the previous owner.
pub fn reassign_doc_quota(
    trigger: On<Insert, SpaceOwner>,
    docs: Query<(&HsdDocId, &SpaceOwner), With<Hsd>>,
    spaces: Query<&Space>,
) {
    let Ok((record, owner)) = docs.get(trigger.entity) else {
        return;
    };
    let Ok(space) = spaces.get(owner.0) else {
        return;
    };
    reassign_document_in_space(record.0, unavi_space::membership::space_doc_id(space));
}

pub fn forget_space_quota(trigger: On<Remove, Space>, spaces: Query<&Space>) {
    if let Ok(space) = spaces.get(trigger.entity) {
        forget_space(space.0);
    }
}

pub fn forget_peer_quota(trigger: On<Remove, Peer>, peers: Query<&Peer>) {
    if let Ok(peer) = peers.get(trigger.entity) {
        forget_peer(NamespaceId::from(peer.0.id.as_bytes()));
    }
}
