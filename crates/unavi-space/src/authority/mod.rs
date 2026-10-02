//! Who authors a document, and what that lets it do.
//!
//! A document's author is the DID holding its namespace write key: proven by
//! the authorship claim a pin carries, or this node for a document it minted.
//! `unavi-policy` holds the rules; [`SpaceView`] gathers the facts and asks.

use std::sync::Arc;

use bevy::prelude::*;
use bevy_async::{
    AsyncCommands,
    AsyncWorld,
};
use bevy_hsd::document::{
    Hsd,
    HsdDocId,
    Minted,
};
use hsd::id::DocId;
use iroh::EndpointId;
use unavi_policy::{
    Policy,
    ledger::PeerKey,
    permissions::Permissions,
    quota::Quota,
    trust::{
        Trust,
        TrustTable,
    },
};
use xdid::core::did::Did;

use crate::{
    discovery::Peer,
    identity::LocalIdentity,
    membership::{
        Space,
        SpaceId,
        SpaceOwner,
    },
    replication::{
        LocalReplica,
        Replicas,
    },
};

pub mod quota;

/// The policy record, the replicas, and who the local node proved to be:
/// everything needed to judge a document.
#[derive(Resource, Clone)]
pub struct SpaceView {
    policy:      Policy,
    replicas:    Replicas,
    identity:    LocalIdentity,
    me:          EndpointId,
    trust:       TrustTable,
    async_world: AsyncWorld,
}

impl SpaceView {
    #[must_use]
    pub fn new(
        policy: Policy,
        replicas: Replicas,
        identity: LocalIdentity,
        me: EndpointId,
        trust: TrustTable,
        async_world: AsyncWorld,
    ) -> Self {
        replicas
            .attribution()
            .set_local(identity.identity.did().clone(), trust.clone());
        Self {
            policy,
            replicas,
            identity,
            me,
            trust,
            async_world,
        }
    }

    /// A command builder that sends to the world this view belongs to.
    #[must_use]
    pub fn commands(&self) -> AsyncCommands {
        self.async_world.commands()
    }

    /// This node's own pins, holds and session writes.
    #[must_use]
    pub fn local(&self) -> LocalReplica {
        LocalReplica::new(self.me, self.identity.clone(), self.async_world.clone())
    }

    /// The space `doc` belongs to.
    ///
    /// Either the space it was registered into, or the space its pin names. A
    /// prefab instance answers with its host's, since it has neither of its
    /// own.
    #[must_use]
    pub fn space_of(&self, doc: DocId) -> Option<SpaceId> {
        space_of(&self.policy, &self.replicas, doc)
    }

    /// How far `peer` is trusted, judged by the DID it proved. A peer that
    /// proved none is anonymous.
    #[must_use]
    pub fn trust_of(&self, peer: EndpointId) -> Trust {
        if peer == self.me {
            return Trust::Myself;
        }
        self.trust.of_peer(peer, &self.identity.bindings)
    }

    /// How far content `did` authors is trusted.
    #[must_use]
    pub fn trust_of_did(&self, did: &Did) -> Trust {
        if did == self.identity.identity.did() {
            Trust::Myself
        } else {
            self.trust.of_did(did)
        }
    }

    /// Who authored `doc`, resolved through the document that composed it.
    ///
    /// `None` where nothing proved an author, such as a space's own content;
    /// the caller reads that as the guest floor, never as this node.
    #[must_use]
    pub fn author(&self, doc: DocId) -> Option<Did> {
        self.replicas.author(self.policy.root(doc))
    }

    /// Whether this node authored `doc`.
    #[must_use]
    pub fn is_mine(&self, doc: DocId) -> bool {
        self.author(doc).as_ref() == Some(self.identity.identity.did())
    }

    /// What `doc` may call, from how far its author is trusted.
    #[must_use]
    pub fn permissions(&self, doc: DocId) -> Permissions {
        let trust = self
            .author(doc)
            .map_or(Trust::Guest, |did| self.trust_of_did(&did));
        Permissions::for_trust(trust)
    }

    /// `doc`'s quota, charging its author, its space, or the unattributed
    /// budget.
    #[must_use]
    pub fn document_quota(&self, doc: DocId) -> Arc<Quota> {
        let author = self.author(doc);
        self.replicas
            .attribution()
            .document_quota(doc, self.space_of(doc), author.as_ref())
    }

    #[must_use]
    pub const fn me(&self) -> EndpointId {
        self.me
    }

    /// This node's own DID.
    #[must_use]
    pub fn did(&self) -> &Did {
        self.identity.identity.did()
    }

    #[must_use]
    pub const fn policy(&self) -> &Policy {
        &self.policy
    }

    #[must_use]
    pub const fn replicas(&self) -> &Replicas {
        &self.replicas
    }

    #[must_use]
    pub const fn identity(&self) -> &LocalIdentity {
        &self.identity
    }

    #[must_use]
    pub const fn trust(&self) -> &TrustTable {
        &self.trust
    }
}

pub(crate) fn space_of(policy: &Policy, replicas: &Replicas, doc: DocId) -> Option<SpaceId> {
    // The document's own registered space first: a script-minted child of the
    // shell states one for itself while its host belongs to none, and the
    // host-chain fallback inside `registered_space` alone would miss it.
    let root = policy.root(doc);
    policy
        .registered_space(doc)
        .map(SpaceId::of_doc)
        .or_else(|| replicas.space_of(doc))
        .or_else(|| replicas.space_of(root))
}

/// Records a document this node minted as its own.
pub fn record_minted(
    trigger: On<Insert, Minted>,
    docs: Query<&HsdDocId, With<Minted>>,
    replicas: Res<Replicas>,
) {
    if let Ok(doc) = docs.get(trigger.entity) {
        replicas.record_minted(doc.0);
    }
}

/// Forgets a minted document that left the world.
pub fn forget_minted(trigger: On<Remove, Minted>, docs: Query<&HsdDocId>, replicas: Res<Replicas>) {
    if let Ok(doc) = docs.get(trigger.entity) {
        replicas.forget_minted(doc.0);
    }
}

/// Repoints a document's quota at its owner when it joins or changes space,
/// migrating standing usage off the previous owner.
pub fn reassign_doc_quota(
    trigger: On<Insert, SpaceOwner>,
    docs: Query<(&HsdDocId, &SpaceOwner), With<Hsd>>,
    spaces: Query<&Space>,
    replicas: Res<Replicas>,
) {
    let Ok((record, owner)) = docs.get(trigger.entity) else {
        return;
    };
    let Ok(space) = spaces.get(owner.0) else {
        return;
    };
    let author = replicas.author(record.0);
    replicas
        .attribution()
        .reassign(record.0, Some(space.id()), author.as_ref());
}

/// Forgets a departed peer's quota, under its endpoint and under the DID it
/// proved, if that binding is still known.
pub fn forget_peer_quota(
    trigger: On<Remove, Peer>,
    peers: Query<&Peer>,
    policy: Res<Policy>,
    identity: Option<Res<LocalIdentity>>,
) {
    let Ok(peer) = peers.get(trigger.entity) else {
        return;
    };
    let id = peer.0.id;
    policy.forget_peer(&PeerKey::Endpoint(id));
    if let Some(did) = identity.and_then(|identity| identity.bindings.did_of(id)) {
        policy.forget_peer(&PeerKey::Did(did));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(seed: u8) -> DocId {
        DocId([seed; 32])
    }

    /// The shell hangs at the app root and belongs to no space, so a document
    /// it mints names its own space. Resolving `space_of` through the host
    /// chain alone loses that, and the beacon never reads as self-authored.
    #[test]
    fn a_child_of_a_spaceless_host_resolves_by_its_own_space() {
        let policy = Policy::new();
        let replicas = Replicas::new(policy.clone());
        let (space, halo, beacon) = (doc(1), doc(2), doc(3));

        policy.update(space, |record| record.space = Some(space));
        policy.update(beacon, |record| {
            record.host = Some(halo);
            record.space = Some(space);
        });

        assert_eq!(
            space_of(&policy, &replicas, beacon),
            Some(SpaceId::of_doc(space)),
            "a script-minted child keeps its own space when its host has none"
        );
    }
}
