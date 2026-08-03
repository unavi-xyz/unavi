use std::sync::Arc;

use iroh_docs::NamespaceId;
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
    state::replicas::owner,
};

/// Quota for a document, attributed to its owning peer or space.
#[must_use]
pub fn document_quota(doc: NamespaceId) -> Arc<Quota> {
    unavi_quota::registry::document_quota(doc, || document_owner(doc))
}

/// Repoints a document's quota at the owner it has within `space`. The space is
/// passed directly so resolution does not race the membership registry.
pub fn reassign_document_in_space(doc: NamespaceId, space: NamespaceId) {
    reassign_document(doc, || Some(space_document_owner(doc, space)));
}

fn document_owner(doc: NamespaceId) -> Option<Arc<Quota>> {
    match doc_space(doc) {
        Some(space) => Some(space_document_owner(doc, space)),
        None => Some(peer_quota(NamespaceId::from(&self_peer_id()?))),
    }
}

fn space_document_owner(doc: NamespaceId, space: NamespaceId) -> Arc<Quota> {
    if space == doc {
        return space_quota(space);
    }
    owner(space, doc).map_or_else(
        || space_quota(space),
        |peer| peer_quota(NamespaceId::from(&peer)),
    )
}
