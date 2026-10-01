//! Which budget a document's charges land in.
//!
//! A document is charged to its author, to its space when it has no author,
//! and to a shared unattributed budget when it has neither. All of them roll
//! up into the node.

use std::sync::Arc;

use hsd::id::DocId;
use parking_lot::RwLock;
use unavi_policy::{
    Policy,
    ledger::PeerKey,
    quota::{
        Quota,
        limits::Limits,
    },
    trust::{
        Trust,
        TrustTable,
    },
};
use xdid::core::did::Did;

use crate::membership::SpaceId;

/// Resolves quotas from facts the caller already holds, so it never reads
/// replicated state and can run under the replica lock.
#[derive(Clone)]
pub struct Attribution {
    policy: Policy,
    local:  Arc<RwLock<Option<Local>>>,
}

/// The local DID and the table judging everyone else. Absent until the
/// identity loads; until then every DID is anonymous.
struct Local {
    did:   Did,
    trust: TrustTable,
}

impl Attribution {
    #[must_use]
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            local: Arc::default(),
        }
    }

    #[must_use]
    pub const fn policy(&self) -> &Policy {
        &self.policy
    }

    /// Names the local DID, so documents it authors are charged as this node's
    /// own.
    pub fn set_local(&self, did: Did, trust: TrustTable) {
        *self.local.write() = Some(Local { did, trust });
    }

    /// The local DID, once the identity has loaded.
    #[must_use]
    pub fn local_did(&self) -> Option<Did> {
        self.local.read().as_ref().map(|local| local.did.clone())
    }

    /// How far content `did` authors is trusted.
    #[must_use]
    pub fn trust_of_did(&self, did: &Did) -> Trust {
        let Some((me, trust)) = self
            .local
            .read()
            .as_ref()
            .map(|local| (local.did.clone(), local.trust.clone()))
        else {
            return Trust::Anonymous;
        };
        if &me == did {
            Trust::Myself
        } else {
            trust.of_did(did)
        }
    }

    /// `author`'s budget, sized by how far it is trusted and shared by every
    /// device proving that DID.
    #[must_use]
    pub fn author_quota(&self, author: &Did) -> Arc<Quota> {
        let trust = self.trust_of_did(author);
        self.policy
            .peer_quota(&PeerKey::Did(author.clone()), || Limits::for_trust(trust))
    }

    /// The budget `doc` rolls up into. A space's own document always charges
    /// the space.
    #[must_use]
    pub fn owner_quota(
        &self,
        doc: DocId,
        space: Option<SpaceId>,
        author: Option<&Did>,
    ) -> Arc<Quota> {
        let is_space_doc = space.is_some_and(|space| space.doc() == doc);
        match (author, space) {
            (Some(author), _) if !is_space_doc => self.author_quota(author),
            (_, Some(space)) => self.policy.space_quota(space.doc()),
            (_, None) => self.policy.unattributed_quota(),
        }
    }

    /// `doc`'s own quota, created against [`Self::owner_quota`] on first use.
    #[must_use]
    pub fn document_quota(
        &self,
        doc: DocId,
        space: Option<SpaceId>,
        author: Option<&Did>,
    ) -> Arc<Quota> {
        self.policy
            .document_quota(doc, || Some(self.owner_quota(doc, space, author)))
    }

    /// Repoints `doc`'s quota at its owner, migrating standing usage.
    pub fn reassign(&self, doc: DocId, space: Option<SpaceId>, author: Option<&Did>) {
        self.policy
            .reassign_document(doc, || Some(self.owner_quota(doc, space, author)));
    }
}

#[cfg(test)]
mod tests {
    use unavi_local::DeviceStorage;
    use unavi_policy::quota::Stock;

    use super::*;

    fn did(name: &str) -> Did {
        format!("did:web:{name}.example").parse().expect("did")
    }

    #[test]
    fn an_authors_documents_share_one_budget() {
        let attribution = Attribution::new(Policy::new());
        let author = did("alice");
        let space = SpaceId([9; 32]);

        let a = attribution.document_quota(DocId([1; 32]), Some(space), Some(&author));
        let b = attribution.document_quota(DocId([2; 32]), Some(space), Some(&author));
        a.charge(Stock::Prims, 3).expect("charge");
        b.charge(Stock::Prims, 4).expect("charge");

        assert_eq!(attribution.author_quota(&author).usage(Stock::Prims), 7);
    }

    #[test]
    fn the_local_did_is_trusted_as_myself() {
        let attribution = Attribution::new(Policy::new());
        let me = did("me");
        assert_eq!(attribution.trust_of_did(&me), Trust::Anonymous);

        attribution.set_local(me.clone(), TrustTable::new(DeviceStorage::memory()));
        assert_eq!(attribution.trust_of_did(&me), Trust::Myself);
        assert_eq!(attribution.trust_of_did(&did("other")), Trust::Guest);
    }
}
