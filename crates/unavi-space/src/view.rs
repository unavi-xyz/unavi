//! What the local node can see when judging a document.
//!
//! `unavi-policy` holds the rules but cannot resolve who owns a document, since
//! ownership is the oldest pin and pins are replicated state. This is where the
//! two meet: the facts are gathered here and the rules are asked there.

use bevy::prelude::Resource;
use hsd::id::DocId;
use iroh::EndpointId;
use unavi_policy::{
    error::PolicyError,
    owner::Owner,
    registry::Policy,
    standing::Standing,
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

    /// Resolves where `doc` stands. Called twice per write check, and the write
    /// path runs on every prim write.
    #[must_use]
    pub fn standing(&self, doc: DocId) -> Standing {
        let root = self.policy.root(doc);
        let replicated = self.replicas.space_of(root);
        let space = self.policy.registered_space(root).or(replicated);

        let record = self.policy.get(doc);
        let owner = record
            .owner
            .unwrap_or_else(|| self.resolve_owner(root, space, replicated));

        Standing {
            owner,
            space,
            threshold: record.threshold,
            trust: self.trust_of(&owner),
        }
    }

    /// Owner for a document the host stated none for: the space's own
    /// document, then the oldest pin, then the space itself, then — for
    /// something minted here — this node.
    fn resolve_owner(&self, root: DocId, space: Option<DocId>, replicated: Option<DocId>) -> Owner {
        match space {
            Some(space) if root == space => Owner::Space(space),
            Some(space) => self.replicas.owner(space, root).map_or_else(
                || {
                    if replicated.is_none() {
                        // Nothing pins it and it is absent from the replica
                        // index, so it was minted here. A document that *is*
                        // in the index arrived from a peer, and must never
                        // fall back to reading as local.
                        Owner::Peer(self.me)
                    } else {
                        Owner::Space(space)
                    }
                },
                Owner::Peer,
            ),
            None => Owner::Peer(self.me),
        }
    }

    /// Whether `caller` may write `target`.
    pub fn write(&self, caller: DocId, target: DocId) -> Result<(), PolicyError> {
        if caller == target {
            return Ok(());
        }
        self.standing(caller).may_write(&self.standing(target))
    }

    /// Whether `caller` may read `target`.
    pub fn read(&self, caller: DocId, target: DocId) -> Result<(), PolicyError> {
        if caller == target {
            return Ok(());
        }
        self.standing(caller).may_read(&self.standing(target))
    }

    /// Whether `caller` is placed well enough to reach anything outside itself.
    pub fn placed(&self, caller: DocId) -> Result<(), PolicyError> {
        self.standing(caller).placed()
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

    #[must_use]
    pub fn same_space(&self, a: DocId, b: DocId) -> bool {
        match (self.space_of(a), self.space_of(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// Whether `doc`'s owner may reach outside every space.
    #[must_use]
    pub fn crosses_space_boundaries(&self, doc: DocId) -> bool {
        self.standing(doc).owner.crosses_space_boundaries()
    }

    /// The trust to judge a document by, given its owner.
    ///
    /// A space's content is nobody's opinion, so it is judged at the guest
    /// floor; the node's own content at the top, since it is the node's.
    #[must_use]
    pub fn trust_of(&self, owner: &Owner) -> Trust {
        match owner {
            Owner::System => Trust::Myself,
            Owner::Space(_) => Trust::Guest,
            Owner::Peer(peer) => quota::trust_of(Some(self.viewer()), *peer),
        }
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
