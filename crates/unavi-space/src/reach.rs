use hsd::id::DocId;
use iroh_docs::NamespaceId;
use unavi_policy::{
    error::PolicyError,
    reach,
    tier::Tier,
    trust::{
        self,
        Trust,
    },
};

use crate::{
    membership,
    peer::self_peer_id,
    state::replicas,
};

/// Whether `caller` may write `target`.
///
/// The document-to-document question is answered by looking at the owners: a
/// document's authority is a function of the trust its owning peer has, so
/// there is no per-document matrix to populate.
pub fn check_write(caller: DocId, tier: Tier, target: DocId) -> Result<(), PolicyError> {
    if caller == target {
        return Ok(());
    }

    let author = owner_of(caller);
    reach::permits(
        trust_of(author),
        reach::required(target),
        author == owner_of(target),
        tier.crosses_space_boundaries() || membership::same_space(caller, target),
    )
}

/// Whether `caller` may read `target`.
///
/// Reads are open within a space, so membership is the whole gate. A document
/// that wants to be unreadable has to be in a namespace the reader has no id
/// for, which is the only place reading can actually be prevented.
pub fn check_read(caller: DocId, tier: Tier, target: DocId) -> Result<(), PolicyError> {
    if caller == target || tier.crosses_space_boundaries() || membership::same_space(caller, target)
    {
        Ok(())
    } else {
        Err(PolicyError::Reach(
            "documents are not in the same space".into(),
        ))
    }
}

/// The peer whose pin owns `doc`.
///
/// A document with no pin is local — nothing arrives from a peer without one —
/// and so answers with the local peer rather than with absence. Both the
/// unpinned case and the not-in-a-space case have to land there, or the shell
/// would read as a different owner than the props it minted.
fn owner_of(doc: DocId) -> Option<[u8; 32]> {
    membership::doc_space(doc)
        .and_then(|space| replicas::owner(NamespaceId::from(&space.0), NamespaceId::from(&doc.0)))
        .or_else(self_peer_id)
}

fn trust_of(owner: Option<[u8; 32]>) -> Trust {
    match owner {
        None => Trust::Myself,
        Some(peer) if Some(peer) == self_peer_id() => Trust::Myself,
        Some(peer) => trust::of_peer(peer),
    }
}
