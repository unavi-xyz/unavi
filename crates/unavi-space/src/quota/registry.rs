use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdDocId,
};
use unavi_policy::{
    membership::SpaceOwner,
    registry::Policy,
    space::Space,
};

use crate::{
    peer::Peer,
    quota::reassign_document_in_space,
    state::replicas::Replicas,
    view::SpaceView,
};

/// Repoints a document's quota at its owner when it joins or changes space,
/// migrating standing usage off the previous owner.
///
/// Runs with `viewer: None` until the identity is ready, same as every other
/// quota resolution site — the reassignment must not be skipped just because
/// the view hasn't loaded yet.
pub fn reassign_doc_quota(
    trigger: On<Insert, SpaceOwner>,
    docs: Query<(&HsdDocId, &SpaceOwner), With<Hsd>>,
    spaces: Query<&Space>,
    policy: Res<Policy>,
    replicas: Res<Replicas>,
    view: Option<Res<SpaceView>>,
) {
    let Ok((record, owner)) = docs.get(trigger.entity) else {
        return;
    };
    let Ok(space) = spaces.get(owner.0) else {
        return;
    };
    reassign_document_in_space(
        &policy,
        &replicas,
        view.as_deref().map(SpaceView::viewer),
        record.0,
        space.doc_id(),
    );
}

pub fn forget_space_quota(trigger: On<Remove, Space>, spaces: Query<&Space>, policy: Res<Policy>) {
    if let Ok(space) = spaces.get(trigger.entity) {
        policy.forget_space(space.doc_id());
    }
}

pub fn forget_peer_quota(trigger: On<Remove, Peer>, peers: Query<&Peer>, policy: Res<Policy>) {
    if let Ok(peer) = peers.get(trigger.entity) {
        policy.forget_peer(peer.0.id);
    }
}
