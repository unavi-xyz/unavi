use std::{
    collections::HashMap,
    sync::Arc,
};

use bevy::prelude::Resource;
use hsd::id::DocId;
use iroh::EndpointId;
use parking_lot::RwLock;

use crate::{
    document::DocumentPolicy,
    quota::{
        Quota,
        limits::Limits,
    },
    reach::Reach,
};

/// Longest host chain a lookup will follow before giving up. A cycle is not
/// reachable through the derived-id scheme, so the cap guards a corrupted
/// registry rather than an expected case.
const MAX_HOST_DEPTH: usize = 16;

/// Everything the host has decided about one document.
///
/// One record rather than a registry per field: all of it is keyed by the same
/// document id and read together on the write path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
    pub policy: DocumentPolicy,
    pub reach:  Reach,
    /// The space this document was registered into, ignoring any that only a
    /// peer's pin places.
    pub space:  Option<DocId>,
    /// The document that composed this one in, for a prefab instance. An
    /// instance has an id but no namespace, so its owner and its space are
    /// whatever its host's are.
    pub host:   Option<DocId>,
}

/// What a quota is attributed to.
///
/// A document's charges roll up into the peer that pinned it, or into the space
/// when nobody has, and every scope rolls up into the node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Principal {
    Document(DocId),
    Peer(EndpointId),
    Space(DocId),
}

/// The host's decisions about every document it knows, and the quota each one
/// spends against.
///
/// One value, constructed once per app. Both halves are keyed by document id
/// and share a lifecycle, so a document dropped from one is dropped from both.
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
            node:      Quota::root(Limits::global()),
        }))
    }

    /// What the host decided about `doc`.
    ///
    /// An unregistered document answers [`Record::default`]: silence and the
    /// weakest answer are the same statement.
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

    /// Drops `doc`'s record and its quota, releasing whatever the quota still
    /// held from its owner.
    pub fn forget(&self, doc: DocId) {
        self.0.documents.write().remove(&doc);
        let quota = self.0.quotas.write().remove(&Principal::Document(doc));
        if let Some(quota) = quota {
            quota.set_owner(None);
        }
    }

    /// Drops the space's own record and quota, and unregisters its members
    /// from it.
    ///
    /// The members keep their records: each drops its own when its document id
    /// goes, and a member that outlives the space must not silently regain the
    /// open defaults in the meantime.
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

    /// Drops `peer`'s quota, so the next document it owns re-derives the caps
    /// from its current rung.
    pub fn forget_peer(&self, peer: EndpointId) {
        self.0.quotas.write().remove(&Principal::Peer(peer));
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
    ///
    /// A prefab instance is never pinned and has no namespace, so it is its
    /// host that owns it; resolving through the root is what stops a peer's
    /// instanced content reading as locally authored.
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

    /// The quota every other scope rolls up into.
    #[must_use]
    pub fn node_quota(&self) -> &Arc<Quota> {
        &self.0.node
    }

    #[must_use]
    pub fn space_quota(&self, space: DocId) -> Arc<Quota> {
        self.quota(Principal::Space(space), Limits::space)
    }

    /// A peer's quota under the caps its rung earns, derived on first sight.
    /// [`Self::forget_peer`] is what re-derives them after a rung change.
    #[must_use]
    pub fn peer_quota(&self, peer: EndpointId, limits: impl FnOnce() -> Limits) -> Arc<Quota> {
        self.quota(Principal::Peer(peer), limits)
    }

    /// `doc`'s quota, rolling its charges up into `owner`. An owner-less
    /// document still gets one, so its charges accumulate; it simply does not
    /// roll up past itself.
    pub fn document_quota(
        &self,
        doc: DocId,
        owner: impl FnOnce() -> Option<Arc<Quota>>,
    ) -> Arc<Quota> {
        let principal = Principal::Document(doc);
        if let Some(quota) = self.0.quotas.read().get(&principal) {
            return Arc::clone(quota);
        }
        // Resolved before the write lock is taken: an owner resolver re-enters
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

    /// Gives a document another document spawned a quota rolling up into
    /// whatever the parent rolls up into.
    ///
    /// A parent this registry does not track yet has no owner to inherit, so
    /// the child gets none rather than one resolved from somewhere else.
    pub fn attribute_child_document(&self, doc: DocId, parent: DocId) {
        let owner = self
            .0
            .quotas
            .read()
            .get(&Principal::Document(parent))
            .and_then(|quota| quota.owner());
        self.document_quota(doc, || owner);
    }

    /// Repoints a live document's quota at `owner`, migrating its standing
    /// usage off the previous owner. `owner` is resolved only if the document
    /// is still tracked.
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

    /// A distinct, valid endpoint id per seed. Arbitrary bytes are not a curve
    /// point, so a key has to be derived rather than written down.
    fn peer(seed: u8) -> EndpointId {
        SecretKey::from_bytes(&[seed; 32]).public()
    }

    #[test]
    fn an_unregistered_document_answers_the_weakest_record() {
        let record = Policy::new().get(doc(b"never-registered"));
        assert_eq!(record.policy, DocumentPolicy::untrusted());
        assert_eq!(record.reach, Reach::default());
        assert!(record.space.is_none());
    }

    #[test]
    fn a_space_takes_its_documents_registrations_with_it() {
        let policy = Policy::new();
        let (space, member, other) = (doc(b"space"), doc(b"member"), doc(b"other"));
        policy.update(space, |r| r.space = Some(space));
        policy.update(member, |r| r.space = Some(space));
        policy.update(other, |r| r.reach = Reach::own_only());

        policy.forget_space(space);

        assert!(policy.get(member).space.is_none());
        assert_eq!(
            policy.get(other).reach,
            Reach::own_only(),
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
            "a prefab inside a prefab stands where its host stands"
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

    /// Each document is capped, but the owning peer caps the aggregate: enough
    /// full documents exhaust the peer budget while each stays within its own.
    #[test]
    fn kv_memory_rolls_up_to_peer_across_docs() {
        let policy = Policy::new();
        let peer = policy.peer_quota(peer(7), Limits::peer);
        let cap = |limits: Limits| *limits.stock.get(&Stock::KvMemory).expect("caps kv memory");
        let (doc_cap, peer_cap) = (cap(Limits::document()), cap(Limits::peer()));

        for i in 0..peer_cap / doc_cap {
            let quota = policy.document_quota(doc(&i.to_le_bytes()), || Some(Arc::clone(&peer)));
            quota
                .try_charge(Stock::KvMemory, doc_cap)
                .expect("doc fits within the peer budget");
        }

        let overflow = policy.document_quota(doc(b"overflow"), || Some(Arc::clone(&peer)));
        assert!(matches!(
            overflow.try_charge(Stock::KvMemory, doc_cap),
            Err(QuotaError::Stock(Stock::KvMemory))
        ));
    }

    #[test]
    fn forgetting_a_document_releases_what_it_held_from_its_owner() {
        let policy = Policy::new();
        let peer = policy.peer_quota(peer(9), Limits::peer);
        let id = doc(b"released");

        let quota = policy.document_quota(id, || Some(Arc::clone(&peer)));
        quota.try_charge(Stock::Prims, 100).expect("charge");
        assert_eq!(peer.usage(Stock::Prims), 100);

        policy.forget(id);

        assert_eq!(
            peer.usage(Stock::Prims),
            0,
            "a document's charges must not outlive its record"
        );
    }
}
