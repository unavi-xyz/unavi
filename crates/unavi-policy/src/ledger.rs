//! The host's record of every document it holds, and the quota each spends
//! against.

use std::{
    collections::HashMap,
    sync::Arc,
};

use bevy::ecs::resource::Resource;
use hsd::id::DocId;
use iroh::EndpointId;
use parking_lot::RwLock;
use unavi_identity::auth::Bindings;
use xdid::core::did::Did;

use crate::quota::{
    Quota,
    limits::Limits,
};

/// Longest host chain a lookup follows before giving up.
const MAX_HOST_DEPTH: usize = 16;

/// What the host has decided about one document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
    /// The space this document was registered into locally.
    pub space: Option<DocId>,
    /// The document that composed this one in: the host of a reference site,
    /// or the document whose script created it. Authorship and the space both
    /// resolve through it.
    pub host:  Option<DocId>,
}

/// Who a peer's quota belongs to: the DID it proved, so every device of one
/// identity shares a budget and a reconnect does not refill it, or its
/// endpoint until it proves one.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PeerKey {
    Did(Did),
    Endpoint(EndpointId),
}

impl PeerKey {
    /// `peer`'s key as `bindings` currently knows it.
    #[must_use]
    pub fn of(peer: EndpointId, bindings: &Bindings) -> Self {
        bindings
            .did_of(peer)
            .map_or(Self::Endpoint(peer), Self::Did)
    }
}

/// What a quota is attributed to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Principal {
    Document(DocId),
    Peer(PeerKey),
    Space(DocId),
}

/// Every document's record and quota, keyed by document id. One value per
/// app.
#[derive(Resource, Clone)]
pub struct Policy(Arc<Inner>);

struct Inner {
    documents: RwLock<HashMap<DocId, Record>>,
    quotas:    RwLock<HashMap<Principal, Arc<Quota>>>,
    node:      Arc<Quota>,
}

impl Default for Policy {
    fn default() -> Self {
        Self::new()
    }
}

impl Policy {
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(Inner {
            documents: RwLock::default(),
            quotas:    RwLock::default(),
            node:      Quota::new(Limits::global(), None),
        }))
    }

    /// What the host decided about `doc`. An unregistered document answers
    /// [`Record::default`].
    #[must_use]
    pub fn get(&self, doc: DocId) -> Record {
        self.0
            .documents
            .read()
            .get(&doc)
            .copied()
            .unwrap_or_default()
    }

    pub fn update(&self, doc: DocId, f: impl FnOnce(&mut Record)) {
        f(self.0.documents.write().entry(doc).or_default());
    }

    /// Drops `doc`'s record and quota, releasing what the quota held.
    pub fn forget_document(&self, doc: DocId) {
        self.0.documents.write().remove(&doc);
        let quota = self.0.quotas.write().remove(&Principal::Document(doc));
        if let Some(quota) = quota {
            quota.set_owner(None);
        }
    }

    /// Drops the space's record and quota, and clears it from its members'
    /// records. The members keep their records.
    pub fn forget_space(&self, space: DocId) {
        let mut docs = self.0.documents.write();
        docs.remove(&space);
        for record in docs.values_mut() {
            if record.space == Some(space) {
                record.space = None;
            }
        }
        drop(docs);
        self.0.quotas.write().remove(&Principal::Space(space));
    }

    /// Drops `peer`'s quota. Documents already charging it keep it; the next
    /// lookup starts a fresh one.
    pub fn forget_peer(&self, peer: &PeerKey) {
        self.0.quotas.write().remove(&Principal::Peer(peer.clone()));
    }

    /// Applies `limits` to `peer`'s quota in place, reaching every document
    /// already charging it. A peer with no quota yet gets `limits` on first
    /// sight anyway.
    pub fn retrust_peer(&self, peer: &PeerKey, limits: Limits) {
        let quota = self
            .0
            .quotas
            .read()
            .get(&Principal::Peer(peer.clone()))
            .map(Arc::clone);
        if let Some(quota) = quota {
            quota.set_limits(limits);
        }
    }

    /// Every document registered into `space`.
    #[must_use]
    pub fn documents_in(&self, space: DocId) -> Vec<DocId> {
        self.0
            .documents
            .read()
            .iter()
            .filter(|(_, record)| record.space == Some(space))
            .map(|(doc, _)| *doc)
            .collect()
    }

    /// The document at the top of `doc`'s host chain.
    #[must_use]
    pub fn root(&self, doc: DocId) -> DocId {
        let docs = self.0.documents.read();
        let mut at = doc;
        for _ in 0..MAX_HOST_DEPTH {
            match docs.get(&at).and_then(|record| record.host) {
                Some(host) if host != at => at = host,
                _ => break,
            }
        }
        at
    }

    /// The space `doc` was registered into, following the host chain.
    #[must_use]
    pub fn registered_space(&self, doc: DocId) -> Option<DocId> {
        let root = self.root(doc);
        let docs = self.0.documents.read();
        docs.get(&doc)
            .and_then(|record| record.space)
            .or_else(|| docs.get(&root).and_then(|record| record.space))
    }

    #[must_use]
    pub fn space_quota(&self, space: DocId) -> Arc<Quota> {
        self.quota(Principal::Space(space), Limits::space)
    }

    /// A peer's quota, with `limits` applied on first sight. Changed by
    /// [`Self::retrust_peer`].
    #[must_use]
    pub fn peer_quota(&self, peer: &PeerKey, limits: impl FnOnce() -> Limits) -> Arc<Quota> {
        self.quota(Principal::Peer(peer.clone()), limits)
    }

    /// `doc`'s quota, rolling its charges up into `owner`. An owner-less
    /// document gets one that does not roll up.
    pub fn document_quota(
        &self,
        doc: DocId,
        owner: impl FnOnce() -> Option<Arc<Quota>>,
    ) -> Arc<Quota> {
        let principal = Principal::Document(doc);
        if let Some(quota) = self.0.quotas.read().get(&principal) {
            return Arc::clone(quota);
        }
        // Resolved before the write lock is taken. An owner resolver re-enters
        // the caller's own state, so holding this lock across it would close a
        // cycle between the two.
        let owner = owner();
        let mut quotas = self.0.quotas.write();
        Arc::clone(
            quotas
                .entry(principal)
                .or_insert_with(|| Quota::new(Limits::document(), owner)),
        )
    }

    /// Gives `doc` a quota rolling up into whatever `parent` rolls up into.
    /// An untracked `parent` leaves the child with no owner.
    pub fn attribute_child_document(&self, doc: DocId, parent: DocId) {
        let owner = self
            .0
            .quotas
            .read()
            .get(&Principal::Document(parent))
            .and_then(|quota| quota.owner());
        self.document_quota(doc, || owner);
    }

    /// Repoints `doc`'s quota at `owner`, migrating its standing usage.
    /// `owner` is resolved only if `doc` is still tracked.
    pub fn reassign_document(&self, doc: DocId, owner: impl FnOnce() -> Option<Arc<Quota>>) {
        let quota = self
            .0
            .quotas
            .read()
            .get(&Principal::Document(doc))
            .map(Arc::clone);
        if let Some(quota) = quota {
            quota.set_owner(owner());
        }
    }

    fn quota(&self, principal: Principal, limits: impl FnOnce() -> Limits) -> Arc<Quota> {
        if let Some(quota) = self.0.quotas.read().get(&principal) {
            return Arc::clone(quota);
        }
        let limits = limits();
        let mut quotas = self.0.quotas.write();
        Arc::clone(
            quotas
                .entry(principal)
                .or_insert_with(|| Quota::new(limits, Some(Arc::clone(&self.0.node)))),
        )
    }
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;

    use super::*;
    use crate::quota::{
        QuotaError,
        Stock,
    };

    fn doc(seed: &[u8]) -> DocId {
        DocId(*blake3::hash(seed).as_bytes())
    }

    /// Arbitrary bytes are not a curve point, so the key is derived.
    fn peer(seed: u8) -> EndpointId {
        SecretKey::from_bytes(&[seed; 32]).public()
    }

    #[test]
    fn an_unregistered_document_answers_the_empty_record() {
        assert_eq!(
            Policy::new().get(doc(b"never-registered")),
            Record::default()
        );
    }

    #[test]
    fn a_space_takes_its_documents_registrations_with_it() {
        let policy = Policy::new();
        let (space, member, other) = (doc(b"space"), doc(b"member"), doc(b"other"));
        policy.update(space, |r| r.space = Some(space));
        policy.update(member, |r| r.space = Some(space));
        policy.update(other, |r| r.host = Some(space));

        policy.forget_space(space);

        assert!(policy.get(member).space.is_none());
        assert_eq!(
            policy.get(other).host,
            Some(space),
            "unloading one space must not clear an unrelated document"
        );
    }

    #[test]
    fn an_instance_resolves_through_its_host() {
        let policy = Policy::new();
        let (space, host, instance, nested) = (
            doc(b"space"),
            doc(b"host"),
            doc(b"instance"),
            doc(b"nested"),
        );
        policy.update(host, |r| r.space = Some(space));
        policy.update(instance, |r| r.host = Some(host));
        policy.update(nested, |r| r.host = Some(instance));

        assert_eq!(policy.root(nested), host);
        assert_eq!(
            policy.registered_space(nested),
            Some(space),
            "a reference inside a reference stands where its host stands"
        );
    }

    #[test]
    fn a_host_cycle_terminates() {
        let policy = Policy::new();
        let (a, b) = (doc(b"cycle-a"), doc(b"cycle-b"));
        policy.update(a, |r| r.host = Some(b));
        policy.update(b, |r| r.host = Some(a));

        let _ = policy.root(a);
    }

    /// The owning peer caps the aggregate, not just each document.
    #[test]
    fn session_memory_rolls_up_to_peer_across_docs() {
        let policy = Policy::new();
        let peer = policy.peer_quota(&PeerKey::Endpoint(peer(7)), Limits::peer);
        let cap = |limits: Limits| {
            *limits
                .stock
                .get(&Stock::SessionMemory)
                .expect("caps session memory")
        };
        let (doc_cap, peer_cap) = (cap(Limits::document()), cap(Limits::peer()));

        for i in 0..peer_cap / doc_cap {
            let quota = policy.document_quota(doc(&i.to_le_bytes()), || Some(Arc::clone(&peer)));
            quota
                .charge(Stock::SessionMemory, doc_cap)
                .expect("doc fits within the peer budget");
        }

        let overflow = policy.document_quota(doc(b"overflow"), || Some(Arc::clone(&peer)));
        assert!(matches!(
            overflow.charge(Stock::SessionMemory, doc_cap),
            Err(QuotaError::Stock(Stock::SessionMemory))
        ));
    }

    #[test]
    fn forgetting_a_document_releases_what_it_held_from_its_owner() {
        let policy = Policy::new();
        let peer = policy.peer_quota(&PeerKey::Endpoint(peer(9)), Limits::peer);
        let id = doc(b"released");

        let quota = policy.document_quota(id, || Some(Arc::clone(&peer)));
        quota.charge(Stock::Prims, 100).expect("charge");
        assert_eq!(peer.usage(Stock::Prims), 100);

        policy.forget_document(id);

        assert_eq!(
            peer.usage(Stock::Prims),
            0,
            "a document's charges must not outlive its record"
        );
    }
}
