use std::sync::Arc;

pub mod registry;

use hsd::id::DocId;
use iroh_docs::NamespaceId;
use unavi_policy::check::{
    space_of,
    trust_of,
};
use unavi_quota::{
    Quota,
    registry::{
        peer_quota,
        peer_quota_with,
        reassign_document,
        space_quota,
    },
};

use crate::{
    peer::self_peer_id,
    state::replicas::owner,
};

/// The quota registry keys by 32 opaque bytes, which a document id and a
/// namespace id equally are.
fn key(doc: DocId) -> NamespaceId {
    NamespaceId::from(&doc.0)
}

/// Quota attributed to the document's owning peer or space.
#[must_use]
pub fn document_quota(doc: DocId) -> Arc<Quota> {
    unavi_quota::registry::document_quota(key(doc), || document_owner(doc))
}

/// Repoints a document's quota at the owner it has within `space`. The space is
/// passed directly so resolution does not race the membership registry.
pub fn reassign_document_in_space(doc: DocId, space: DocId) {
    reassign_document(key(doc), || Some(space_document_owner(doc, space)));
}

fn document_owner(doc: DocId) -> Option<Arc<Quota>> {
    match space_of(doc) {
        Some(space) => Some(space_document_owner(doc, space)),
        None => Some(peer_quota(NamespaceId::from(&self_peer_id()?))),
    }
}

fn space_document_owner(doc: DocId, space: DocId) -> Arc<Quota> {
    if space == doc {
        return space_quota(key(space));
    }
    owner(key(space), key(doc)).map_or_else(
        || space_quota(key(space)),
        |peer| {
            // A peer's budget is scaled by how far they are trusted, so a
            // griefer hits a ceiling before anything visible happens and a
            // friend never notices the system exists.
            peer_quota_with(NamespaceId::from(&peer), || {
                unavi_policy::limits::for_trust(trust_of(Some(peer)))
            })
        },
    )
}
