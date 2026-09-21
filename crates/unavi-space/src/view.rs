//! What the local node can see when judging a document.
//!
//! `unavi-policy` holds the rules but cannot resolve who authored a document,
//! since authorship is replicated state. This is where the two meet: the facts
//! are gathered here and the rules are asked there.

use bevy::prelude::Resource;
use hsd::id::DocId;
use iroh::EndpointId;
use unavi_policy::{
    permissions::Permissions,
    registry::Policy,
    trust::{
        Trust,
        TrustTable,
    },
};

use crate::{
    identity::LocalIdentity,
    quota::{
        self,
        Viewer,
    },
    state::replicas::Replicas,
};

/// The policy record, who pins a document, and who the local writer proved to
/// be.
#[derive(Resource, Clone)]
pub struct SpaceView {
    policy:   Policy,
    replicas: Replicas,
    identity: LocalIdentity,
    me:       EndpointId,
    trust:    TrustTable,
}

impl SpaceView {
    #[must_use]
    pub const fn new(
        policy: Policy,
        replicas: Replicas,
        identity: LocalIdentity,
        me: EndpointId,
        trust: TrustTable,
    ) -> Self {
        Self {
            policy,
            replicas,
            identity,
            me,
            trust,
        }
    }

    /// The space `doc` belongs to.
    ///
    /// Either the space it was registered into, or — for a pinned document,
    /// which is namespace-backed and has no local registration — the space some
    /// peer's pin names. A prefab instance answers with its host's, since it
    /// has neither of its own.
    #[must_use]
    pub fn space_of(&self, doc: DocId) -> Option<DocId> {
        quota::space_of(&self.policy, &self.replicas, doc)
    }

    /// The subset of this view the quota resolver can hold. See [`Viewer`].
    #[must_use]
    pub fn viewer(&self) -> Viewer<'_> {
        Viewer {
            me:       self.me,
            bindings: &self.identity.bindings,
            trust:    &self.trust,
        }
    }

    /// How far `peer` is trusted, judged against the identity the local writer
    /// proved. A peer that proved no DID is a guest.
    #[must_use]
    pub fn trust_of(&self, peer: EndpointId) -> Trust {
        quota::trust_of(Some(self.viewer()), peer)
    }

    /// Who authored `doc`, resolved through the document that composed it: the
    /// peer whose pin owns the root, or this node for something minted here.
    ///
    /// `None` where nothing answers — a document present in a space that no
    /// peer's pin claims — and the caller reads that as the guest floor.
    #[must_use]
    pub fn author(&self, doc: DocId) -> Option<EndpointId> {
        let root = self.policy.root(doc);
        let Some(space) = self.space_of(root) else {
            // Absent from every space and from the replica index, so it was
            // minted here. A document that *is* in the index arrived from a
            // peer, and must never fall back to reading as local.
            return Some(self.me);
        };
        self.replicas.owner(space, root)
    }

    /// What `doc` may call, from how far its author is trusted.
    #[must_use]
    pub fn permissions(&self, doc: DocId) -> Permissions {
        let trust = self
            .author(doc)
            .map_or(Trust::Guest, |peer| self.trust_of(peer));
        Permissions::for_trust(trust)
    }

    #[must_use]
    pub const fn me(&self) -> EndpointId {
        self.me
    }

    /// This node's own DID.
    #[must_use]
    pub fn did(&self) -> String {
        self.identity.identity.did().to_string()
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
