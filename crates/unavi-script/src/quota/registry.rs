use std::{
    collections::HashMap,
    sync::{
        Arc,
        LazyLock,
    },
};

use blake3::Hash;
use parking_lot::RwLock;
use unavi_space::{
    membership::doc_space,
    peer::self_peer_id,
    state::owner::doc_owner,
};

use crate::quota::{
    Quota,
    limits::Limits,
};

static GLOBAL_QUOTA: LazyLock<Arc<Quota>> = LazyLock::new(|| Quota::root(Limits::global()));
static SPACE_QUOTAS: LazyLock<RwLock<HashMap<Hash, Arc<Quota>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static PEER_QUOTAS: LazyLock<RwLock<HashMap<Hash, Arc<Quota>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static DOC_QUOTAS: LazyLock<RwLock<HashMap<Hash, Arc<Quota>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn space_quota(space: Hash) -> Arc<Quota> {
    let mut map = SPACE_QUOTAS.write();
    Arc::clone(
        map.entry(space)
            .or_insert_with(|| Quota::new(Limits::space(), Some(Arc::clone(&GLOBAL_QUOTA)))),
    )
}

fn peer_quota(peer: Hash) -> Arc<Quota> {
    let mut map = PEER_QUOTAS.write();
    Arc::clone(
        map.entry(peer)
            .or_insert_with(|| Quota::new(Limits::peer(), Some(Arc::clone(&GLOBAL_QUOTA)))),
    )
}

/// Resolves a document's owner, within a known space.
fn space_document_owner(doc: Hash, space: Hash) -> Arc<Quota> {
    if space == doc {
        return space_quota(space);
    }
    doc_owner(space, doc).map_or_else(|| space_quota(space), |peer| peer_quota(Hash::from(peer)))
}

/// Resolves a document's owner.
/// A document outside any space is owned by the local peer.
fn document_owner(doc: Hash) -> Option<Arc<Quota>> {
    match doc_space(doc) {
        Some(space) => Some(space_document_owner(doc, space)),
        None => Some(peer_quota(Hash::from(self_peer_id()?))),
    }
}

/// Quota for a document, rolling its charges up into the owning space or peer.
/// Memoized.
pub fn document_quota(doc: Hash) -> Arc<Quota> {
    if let Some(quota) = DOC_QUOTAS.read().get(&doc) {
        return Arc::clone(quota);
    }
    let Some(owner) = document_owner(doc) else {
        return Quota::new(Limits::document(), None);
    };
    let mut map = DOC_QUOTAS.write();
    if let Some(quota) = map.get(&doc) {
        return Arc::clone(quota);
    }
    let quota = Quota::new(Limits::document(), Some(owner));
    map.insert(doc, Arc::clone(&quota));
    quota
}

/// Quota for a document spawned by another, inheriting the parent's owner so it
/// rolls up into the same space or peer.
pub fn child_document_quota(doc: Hash, parent: Hash) -> Arc<Quota> {
    if let Some(quota) = DOC_QUOTAS.read().get(&doc) {
        return Arc::clone(quota);
    }
    let owner = document_quota(parent).owner();
    let mut map = DOC_QUOTAS.write();
    if let Some(quota) = map.get(&doc) {
        return Arc::clone(quota);
    }
    let quota = Quota::new(Limits::document(), owner);
    map.insert(doc, Arc::clone(&quota));
    quota
}

/// Repoints a live document's quota at the owner it has within `space`.
///
/// Migrates standing usage off the previous owner. Call when a document joins
/// or changes space; the space is passed directly so resolution does not race
/// the membership registry.
pub fn reassign_document_in_space(doc: Hash, space: Hash) {
    let Some(quota) = DOC_QUOTAS.read().get(&doc).cloned() else {
        return;
    };
    quota.set_owner(Some(space_document_owner(doc, space)));
}

/// Drops a gone document's memoized quota so the table sheds dead scopes,
/// releasing its standing stock from the owner so a torn-down document leaks
/// none of the space's or peer's budget.
pub fn forget_document(doc: Hash) {
    let quota = DOC_QUOTAS.write().remove(&doc);
    if let Some(quota) = quota {
        quota.set_owner(None);
    }
}

/// Drops a departed space's memoized quota. Any documents still holding it keep
/// charging until they too are forgotten; only the table entry is shed.
pub fn forget_space(space: Hash) {
    SPACE_QUOTAS.write().remove(&space);
}

/// Drops a disconnected peer's memoized quota, shedding the table entry the
/// same way [`forget_space`] does.
pub fn forget_peer(peer: Hash) {
    PEER_QUOTAS.write().remove(&peer);
}
