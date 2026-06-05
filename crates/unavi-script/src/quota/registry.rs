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
static USER_QUOTAS: LazyLock<RwLock<HashMap<Hash, Arc<Quota>>>> =
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

fn user_quota(peer: Hash) -> Arc<Quota> {
    let mut map = USER_QUOTAS.write();
    Arc::clone(
        map.entry(peer)
            .or_insert_with(|| Quota::new(Limits::user(), Some(Arc::clone(&GLOBAL_QUOTA)))),
    )
}

/// Resolves a document's single owner: the space itself for a space's root
/// document or any document with no recorded peer (the neutral owner),
/// otherwise the recorded peer. A document outside any space is owned by the
/// local user.
///
/// `None` only while the local peer id is still unknown at startup, leaving a
/// local document uncached so it is reattributed once the id arrives.
fn document_owner(doc: Hash) -> Option<Arc<Quota>> {
    let Some(space) = doc_space(doc) else {
        return Some(user_quota(Hash::from(self_peer_id()?)));
    };
    if space == doc {
        return Some(space_quota(space));
    }
    Some(
        doc_owner(space, doc)
            .map_or_else(|| space_quota(space), |peer| user_quota(Hash::from(peer))),
    )
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

/// Repoints a live document's quota at its current owner, migrating standing
/// usage. Call when ownership changes, as it does when a space syncs a new
/// owner over the network.
pub fn reassign_document(doc: Hash) {
    let Some(quota) = DOC_QUOTAS.read().get(&doc).cloned() else {
        return;
    };
    if let Some(owner) = document_owner(doc) {
        quota.set_owner(Some(owner));
    }
}

/// Drops a gone document's memoized quota so the table sheds dead scopes,
/// releasing its standing stock from the owner so a torn-down document leaks
/// none of the space's or user's budget.
pub fn forget_document(doc: Hash) {
    let quota = DOC_QUOTAS.write().remove(&doc);
    if let Some(quota) = quota {
        quota.set_owner(None);
    }
}
