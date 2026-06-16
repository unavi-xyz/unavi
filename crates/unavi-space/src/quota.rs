use std::sync::Arc;

use blake3::Hash;
use unavi_quota::{
    Quota,
    registry::{
        peer_quota,
        reassign_document,
        space_quota,
    },
};

use crate::{
    membership::doc_space,
    peer::self_peer_id,
    state::owner::doc_owner,
};

/// Quota for a document, attributed to its owning peer or space.
#[must_use]
pub fn document_quota(doc: Hash) -> Arc<Quota> {
    unavi_quota::registry::document_quota(doc, || document_owner(doc))
}

/// Repoints a document's quota at the owner it has within `space`. The space is
/// passed directly so resolution does not race the membership registry.
pub fn reassign_document_in_space(doc: Hash, space: Hash) {
    reassign_document(doc, || Some(space_document_owner(doc, space)));
}

fn document_owner(doc: Hash) -> Option<Arc<Quota>> {
    match doc_space(doc) {
        Some(space) => Some(space_document_owner(doc, space)),
        None => Some(peer_quota(Hash::from(self_peer_id()?))),
    }
}

fn space_document_owner(doc: Hash, space: Hash) -> Arc<Quota> {
    if space == doc {
        return space_quota(space);
    }
    doc_owner(space, doc).map_or_else(|| space_quota(space), |peer| peer_quota(Hash::from(peer)))
}
