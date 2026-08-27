//! Where a document stands, and therefore what it may reach.
//!
//! `unavi-policy` holds the rules but cannot resolve who owns a document, since
//! ownership is the oldest pin and pins are replicated state. This is where the
//! two meet: the facts are gathered here and the rules are asked there.

use hsd::id::DocId;
use iroh::EndpointId;
use iroh_docs::NamespaceId;
use unavi_policy::{
    error::PolicyError,
    reach::Standing,
    registry::Policy,
    tier::Tier,
    trust::{
        self,
        Trust,
    },
};

use crate::{
    peer::self_peer_id,
    state::replicas,
};

/// Resolves where `doc` stands. Called twice per write check, and the write
/// path runs on every prim write.
#[must_use]
pub fn standing(policy: &Policy, doc: DocId) -> Standing {
    let root = policy.root(doc);
    let replicated = replicas::space_of(ns(root)).map(id);
    let space = policy.registered_space(root).or(replicated);

    let owner = space
        .and_then(|space| replicas::owner(ns(space), ns(root)))
        .or_else(|| {
            // Nothing pins the root and it is absent from the replica index, so
            // it was minted here. A document that *is* in the index arrived
            // from a peer, and must never fall back to reading as local.
            replicated.is_none().then(self_peer_id).flatten()
        });

    let record = policy.get(doc);
    Standing {
        tier: record.policy.tier,
        reach: record.reach,
        space,
        owner,
        trust: trust_of(owner),
    }
}

/// The space `doc` belongs to.
///
/// Either the space it was registered into, or — for a pinned document, which
/// is namespace-backed and has no local registration — the space some peer's
/// pin names. A prefab instance answers with its host's, since it has neither
/// of its own.
#[must_use]
pub fn space_of(policy: &Policy, doc: DocId) -> Option<DocId> {
    let root = policy.root(doc);
    policy
        .registered_space(root)
        .or_else(|| replicas::space_of(ns(root)).map(id))
}

/// The rung to judge a document by, given the peer that owns it.
#[must_use]
pub fn trust_of(owner: Option<EndpointId>) -> Trust {
    let Some(peer) = owner else {
        return Trust::Guest;
    };
    if self_peer_id() == Some(peer) {
        return Trust::Myself;
    }
    crate::identity::bindings().map_or(Trust::Guest, |bindings| trust::of_peer(peer, &bindings))
}

/// The tier `doc` was loaded at.
#[must_use]
pub fn tier_of(policy: &Policy, doc: DocId) -> Tier {
    policy.get(doc).policy.tier
}

/// Whether `caller` may write `target`.
pub fn write(policy: &Policy, caller: DocId, target: DocId) -> Result<(), PolicyError> {
    if caller == target {
        return Ok(());
    }
    standing(policy, caller).may_write(&standing(policy, target))
}

/// Whether `caller` may read `target`.
pub fn read(policy: &Policy, caller: DocId, target: DocId) -> Result<(), PolicyError> {
    if caller == target {
        return Ok(());
    }
    standing(policy, caller).may_read(&standing(policy, target))
}

/// Whether `caller` is placed well enough to reach anything outside itself.
pub fn placed(policy: &Policy, caller: DocId) -> Result<(), PolicyError> {
    standing(policy, caller).placed()
}

#[must_use]
pub fn same_space(policy: &Policy, a: DocId, b: DocId) -> bool {
    match (space_of(policy, a), space_of(policy, b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// The registry keys by 32 opaque bytes, which a document id and a namespace id
/// equally are.
fn ns(doc: DocId) -> NamespaceId {
    NamespaceId::from(&doc.0)
}

fn id(ns: NamespaceId) -> DocId {
    DocId(*ns.as_bytes())
}
