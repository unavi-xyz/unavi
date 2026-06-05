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
static USER_QUOTAS: LazyLock<RwLock<HashMap<[u8; 32], Arc<Quota>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static DOC_QUOTAS: LazyLock<RwLock<HashMap<Hash, Arc<Quota>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Documents with no resolvable owner share this scope, so the unattributed
/// fringe is still bounded rather than free.
const ANON_USER: [u8; 32] = [0u8; 32];

fn space_quota(space: Hash) -> Arc<Quota> {
    let mut map = SPACE_QUOTAS.write();
    Arc::clone(
        map.entry(space)
            .or_insert_with(|| Quota::root(Limits::space())),
    )
}

fn user_quota(peer: [u8; 32]) -> Arc<Quota> {
    let mut map = USER_QUOTAS.write();
    Arc::clone(
        map.entry(peer)
            .or_insert_with(|| Quota::root(Limits::user())),
    )
}

/// Quota for a top-level document, charging into both its space and the user
/// who owns it. Memoized, so repeated lookups return the same node.
pub fn document_quota(doc: Hash) -> Arc<Quota> {
    if let Some(quota) = DOC_QUOTAS.read().get(&doc) {
        return Arc::clone(quota);
    }
    let mut parents = Vec::new();
    if let Some(space) = doc_space(doc) {
        parents.push(space_quota(space));
        let owner = doc_owner(space, doc)
            .or_else(self_peer_id)
            .unwrap_or(ANON_USER);
        parents.push(user_quota(owner));
    } else {
        parents.push(user_quota(self_peer_id().unwrap_or(ANON_USER)));
    }
    let quota = Quota::new(Limits::document(), parents);
    DOC_QUOTAS.write().insert(doc, Arc::clone(&quota));
    quota
}

/// Quota for a document loaded or created by another, attributed to its parent.
/// Its usage thereby rolls up through the parent into the same space and user.
pub fn child_document_quota(doc: Hash, parent: Hash) -> Arc<Quota> {
    if let Some(quota) = DOC_QUOTAS.read().get(&doc) {
        return Arc::clone(quota);
    }
    let parent_quota = document_quota(parent);
    let quota = Quota::new(Limits::document(), vec![parent_quota]);
    DOC_QUOTAS.write().insert(doc, Arc::clone(&quota));
    quota
}

/// Drops a document's memoized quota once the document is gone, so the table
/// does not accumulate dead scopes.
pub fn forget_document(doc: Hash) {
    DOC_QUOTAS.write().remove(&doc);
}
