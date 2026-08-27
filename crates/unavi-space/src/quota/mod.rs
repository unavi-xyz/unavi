//! Which budget a document's charges land in.
//!
//! A document is attributed to the peer whose pin owns it, or to the space when
//! nobody's pin does, and both roll up into the node. Resolving that needs the
//! replicated pins, which is why it lives here rather than in `unavi-policy`.

use std::sync::Arc;

use hsd::id::DocId;
use iroh::EndpointId;
use iroh_docs::NamespaceId;
use unavi_policy::{
    quota::{
        Quota,
        limits::Limits,
    },
    registry::Policy,
};

pub mod registry;

use crate::{
    check::{
        space_of,
        trust_of,
    },
    peer::self_peer_id,
    state::replicas,
};

/// Quota attributed to the document's owning peer or space.
#[must_use]
pub fn document_quota(policy: &Policy, doc: DocId) -> Arc<Quota> {
    policy.document_quota(doc, || document_owner(policy, doc))
}

/// Repoints a document's quota at the owner it has within `space`. The space is
/// passed directly so resolution does not race the membership registry.
pub fn reassign_document_in_space(policy: &Policy, doc: DocId, space: DocId) {
    policy.reassign_document(doc, || Some(space_document_owner(policy, doc, space)));
}

fn document_owner(policy: &Policy, doc: DocId) -> Option<Arc<Quota>> {
    match space_of(policy, doc) {
        Some(space) => Some(space_document_owner(policy, doc, space)),
        None => Some(peer_quota(policy, self_peer_id()?)),
    }
}

fn space_document_owner(policy: &Policy, doc: DocId, space: DocId) -> Arc<Quota> {
    if space == doc {
        return policy.space_quota(space);
    }
    replicas::owner(ns(space), ns(doc)).map_or_else(
        || policy.space_quota(space),
        |peer| peer_quota(policy, peer),
    )
}

/// A peer's budget, scaled by how far they are trusted, so a griefer hits a
/// ceiling before anything visible happens and a friend never notices the
/// system exists.
fn peer_quota(policy: &Policy, peer: EndpointId) -> Arc<Quota> {
    policy.peer_quota(peer, || Limits::for_trust(trust_of(Some(peer))))
}

fn ns(doc: DocId) -> NamespaceId {
    NamespaceId::from(&doc.0)
}
