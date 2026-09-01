//! Which budget a document's charges land in.
//!
//! A document is attributed to the peer whose pin owns it, or to the space when
//! nobody's pin does, and both roll up into the node. Resolving that needs the
//! replicated pins, which is why it lives here rather than in `unavi-policy`.

use std::sync::Arc;

use hsd::id::DocId;
use iroh::EndpointId;
use unavi_identity::auth::bindings::Bindings;
use unavi_policy::{
    quota::{
        Quota,
        limits::Limits,
    },
    registry::Policy,
    trust::{
        Trust,
        TrustTable,
    },
};

pub mod registry;

use crate::state::replicas::Replicas;

/// Enough of the local identity to size a peer's quota by trust.
///
/// [`crate::view::SpaceView`] holds the same facts and more, but cannot be used
/// here: quota resolution runs inside [`Replicas`], so a view containing that
/// same store would close a cycle. Absent until the identity is ready.
#[derive(Clone, Copy)]
pub struct Viewer<'a> {
    pub me:       EndpointId,
    pub bindings: &'a Bindings,
    pub trust:    &'a TrustTable,
}

pub(crate) fn trust_of(viewer: Option<Viewer>, peer: EndpointId) -> Trust {
    let Some(viewer) = viewer else {
        return Trust::Guest;
    };
    if viewer.me == peer {
        return Trust::Myself;
    }
    viewer.trust.of_peer(peer, viewer.bindings)
}

pub(crate) fn space_of(policy: &Policy, replicas: &Replicas, doc: DocId) -> Option<DocId> {
    let root = policy.root(doc);
    policy
        .registered_space(root)
        .or_else(|| replicas.space_of(root))
}

/// Quota attributed to the document's owning peer or space.
///
/// `viewer` is the local identity, charged for a document not yet associated
/// with any space — the initial guess for a document just registered locally,
/// settled once its pin (if any) resolves an owner.
#[must_use]
pub fn document_quota(
    policy: &Policy,
    replicas: &Replicas,
    viewer: Option<Viewer>,
    doc: DocId,
) -> Arc<Quota> {
    policy.document_quota(doc, || document_owner(policy, replicas, viewer, doc))
}

/// Repoints a document's quota at the owner it has within `space`. The space is
/// passed directly so resolution does not race the membership registry.
pub fn reassign_document_in_space(
    policy: &Policy,
    replicas: &Replicas,
    viewer: Option<Viewer>,
    doc: DocId,
    space: DocId,
) {
    policy.reassign_document(doc, || {
        Some(space_document_owner(policy, replicas, viewer, doc, space))
    });
}

fn document_owner(
    policy: &Policy,
    replicas: &Replicas,
    viewer: Option<Viewer>,
    doc: DocId,
) -> Option<Arc<Quota>> {
    match space_of(policy, replicas, doc) {
        Some(space) => Some(space_document_owner(policy, replicas, viewer, doc, space)),
        None => Some(peer_quota(policy, viewer, viewer?.me)),
    }
}

fn space_document_owner(
    policy: &Policy,
    replicas: &Replicas,
    viewer: Option<Viewer>,
    doc: DocId,
    space: DocId,
) -> Arc<Quota> {
    if space == doc {
        return policy.space_quota(space);
    }
    replicas.owner(space, doc).map_or_else(
        || policy.space_quota(space),
        |peer| peer_quota(policy, viewer, peer),
    )
}

/// A peer's budget, scaled by how far they are trusted, so a griefer hits a
/// ceiling before anything visible happens and a friend never notices the
/// system exists.
fn peer_quota(policy: &Policy, viewer: Option<Viewer>, peer: EndpointId) -> Arc<Quota> {
    policy.peer_quota(peer, || Limits::for_trust(trust_of(viewer, peer)))
}
