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
            .or_insert_with(|| Quota::root(Limits::space())),
    )
}

fn user_quota(peer: Hash) -> Arc<Quota> {
    let mut map = USER_QUOTAS.write();
    Arc::clone(
        map.entry(peer)
            .or_insert_with(|| Quota::root(Limits::user())),
    )
}

/// Quota for a top-level document. A space anchors its own budget and bears no
/// owner; every other document charges its space and owning user. Memoized.
pub fn document_quota(doc: Hash) -> Arc<Quota> {
    if let Some(quota) = DOC_QUOTAS.read().get(&doc) {
        return Arc::clone(quota);
    }
    let Some(parents) = document_parents(doc) else {
        return Quota::new(Limits::document(), Vec::new());
    };
    let mut map = DOC_QUOTAS.write();
    if let Some(quota) = map.get(&doc) {
        return Arc::clone(quota);
    }
    let quota = Quota::new(Limits::document(), parents);
    map.insert(doc, Arc::clone(&quota));
    quota
}

/// `None` while the local peer id is still unknown at startup, leaving the
/// document uncached so it is reattributed once the id arrives.
fn document_parents(doc: Hash) -> Option<Vec<Arc<Quota>>> {
    let Some(space) = doc_space(doc) else {
        return Some(vec![user_quota(Hash::from(self_peer_id()?))]);
    };
    if space == doc {
        return Some(vec![space_quota(space)]);
    }
    let owner = doc_owner(space, doc).or_else(self_peer_id)?;
    Some(vec![space_quota(space), user_quota(Hash::from(owner))])
}

/// Quota for a document loaded or created by another, rolling up through its
/// parent and thereby inheriting the parent's space and owner.
pub fn child_document_quota(doc: Hash, parent: Hash) -> Arc<Quota> {
    if let Some(quota) = DOC_QUOTAS.read().get(&doc) {
        return Arc::clone(quota);
    }
    let parent_quota = document_quota(parent);
    let mut map = DOC_QUOTAS.write();
    if let Some(quota) = map.get(&doc) {
        return Arc::clone(quota);
    }
    let quota = Quota::new(Limits::document(), vec![parent_quota]);
    map.insert(doc, Arc::clone(&quota));
    quota
}

/// Drops a gone document's memoized quota so the table sheds dead scopes.
pub fn forget_document(doc: Hash) {
    DOC_QUOTAS.write().remove(&doc);
}
