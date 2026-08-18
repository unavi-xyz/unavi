use std::{
    collections::HashMap,
    sync::{
        Arc,
        LazyLock,
    },
};

use iroh_docs::NamespaceId;
use parking_lot::RwLock;

use crate::{
    Quota,
    limits::Limits,
};

static GLOBAL_QUOTA: LazyLock<Arc<Quota>> = LazyLock::new(|| Quota::root(Limits::global()));
static SPACE_QUOTAS: LazyLock<RwLock<HashMap<NamespaceId, Arc<Quota>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static PEER_QUOTAS: LazyLock<RwLock<HashMap<NamespaceId, Arc<Quota>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static DOC_QUOTAS: LazyLock<RwLock<HashMap<NamespaceId, Arc<Quota>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[must_use]
pub fn space_quota(space: NamespaceId) -> Arc<Quota> {
    let mut map = SPACE_QUOTAS.write();
    Arc::clone(
        map.entry(space)
            .or_insert_with(|| Quota::new(Limits::space(), Some(Arc::clone(&GLOBAL_QUOTA)))),
    )
}

#[must_use]
pub fn peer_quota(peer: NamespaceId) -> Arc<Quota> {
    peer_quota_with(peer, Limits::peer)
}

/// A peer's quota under caps the caller chooses, resolved only on first sight.
///
/// Memoized like every other scope, so a peer whose rung changes keeps the caps
/// it was first seen at until [`forget_peer`] drops the entry.
pub fn peer_quota_with(peer: NamespaceId, limits: impl FnOnce() -> Limits) -> Arc<Quota> {
    if let Some(quota) = PEER_QUOTAS.read().get(&peer) {
        return Arc::clone(quota);
    }
    let limits = limits();
    let mut map = PEER_QUOTAS.write();
    Arc::clone(
        map.entry(peer)
            .or_insert_with(|| Quota::new(limits, Some(Arc::clone(&GLOBAL_QUOTA)))),
    )
}

/// Memoized quota for a document, rolling its charges up into `owner` (resolved
/// lazily, only on first sight). An owner-less document still memoizes so its
/// charges accumulate; it simply does not roll up.
pub fn document_quota(doc: NamespaceId, owner: impl FnOnce() -> Option<Arc<Quota>>) -> Arc<Quota> {
    if let Some(quota) = DOC_QUOTAS.read().get(&doc) {
        return Arc::clone(quota);
    }
    let owner = owner();
    let mut map = DOC_QUOTAS.write();
    if let Some(quota) = map.get(&doc) {
        return Arc::clone(quota);
    }
    let quota = Quota::new(Limits::document(), owner);
    map.insert(doc, Arc::clone(&quota));
    quota
}

/// Quota for a document spawned by another, inheriting the parent's owner.
pub fn child_document_quota(doc: NamespaceId, parent: NamespaceId) -> Arc<Quota> {
    if let Some(quota) = DOC_QUOTAS.read().get(&doc) {
        return Arc::clone(quota);
    }
    let owner = DOC_QUOTAS.read().get(&parent).and_then(|q| q.owner());
    let mut map = DOC_QUOTAS.write();
    if let Some(quota) = map.get(&doc) {
        return Arc::clone(quota);
    }
    let quota = Quota::new(Limits::document(), owner);
    map.insert(doc, Arc::clone(&quota));
    quota
}

/// Repoints a live document's quota at `owner`, migrating its standing usage
/// off the previous owner. `owner` is resolved only if the document is still
/// tracked.
pub fn reassign_document(doc: NamespaceId, owner: impl FnOnce() -> Option<Arc<Quota>>) {
    let Some(quota) = DOC_QUOTAS.read().get(&doc).cloned() else {
        return;
    };
    quota.set_owner(owner());
}

/// Forgets a document, releasing its standing stock from the owner.
pub fn forget_document(doc: NamespaceId) {
    let quota = DOC_QUOTAS.write().remove(&doc);
    if let Some(quota) = quota {
        quota.set_owner(None);
    }
}

pub fn forget_space(space: NamespaceId) {
    SPACE_QUOTAS.write().remove(&space);
}

pub fn forget_peer(peer: NamespaceId) {
    PEER_QUOTAS.write().remove(&peer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QuotaError,
        Stock,
    };

    fn h(seed: &[u8]) -> NamespaceId {
        NamespaceId::from(blake3::hash(seed).as_bytes())
    }

    fn kv_cap(limits: Limits) -> u64 {
        *limits.stock.get(&Stock::KvMemory).expect("caps kv memory")
    }

    /// Each document is capped, but the owning peer caps the aggregate: enough
    /// full documents exhaust the peer budget while each stays within its own.
    #[test]
    fn kv_memory_rolls_up_to_peer_across_docs() {
        let peer = peer_quota(h(b"kv-rollup-peer"));
        let doc_cap = kv_cap(Limits::document());
        let peer_cap = kv_cap(Limits::peer());

        for i in 0..peer_cap / doc_cap {
            let q = document_quota(h(&i.to_le_bytes()), || Some(Arc::clone(&peer)));
            q.try_charge(Stock::KvMemory, doc_cap)
                .expect("doc fits within the peer budget");
        }

        let overflow = document_quota(h(b"overflow"), || Some(Arc::clone(&peer)));
        assert!(matches!(
            overflow.try_charge(Stock::KvMemory, doc_cap),
            Err(QuotaError::Stock(Stock::KvMemory))
        ));
    }
}
