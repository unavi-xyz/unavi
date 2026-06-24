use blake3::Hash;
use tracing::warn;

use crate::{
    peer::self_peer_id,
    state::peer,
};

/// Claims `doc` for `peer` in the local replica. Only the local peer can author
/// a claim; the network resolves the definitive owner as the latest claimer.
pub fn set_doc_owner(space: Hash, doc: Hash, peer: [u8; 32]) {
    match self_peer_id() {
        Some(me) if me == peer => peer::self_claim(space, doc),
        Some(_) => warn!(doc = %doc, "ignoring ownership claim for a non-local peer"),
        None => warn!(doc = %doc, "cannot claim doc: local peer id unset"),
    }
}

#[must_use]
pub fn doc_owner(space: Hash, doc: Hash) -> Option<[u8; 32]> {
    peer::owner(space, doc)
}

#[must_use]
pub fn is_self_doc_owner(space: Hash, doc: Hash) -> bool {
    peer::is_self_owner(space, doc)
}

#[cfg(test)]
mod tests {
    use parking_lot::MutexGuard;

    use super::*;
    use crate::peer::set_self_peer_id;

    fn setup(peer: [u8; 32]) -> MutexGuard<'static, ()> {
        let guard = peer::TEST_LOCK.lock();
        peer::reset();
        set_self_peer_id(peer);
        guard
    }

    #[test]
    fn self_claim_sets_owner() {
        let peer = [7u8; 32];
        let _g = setup(peer);
        let space = blake3::hash(b"owner-space");
        let doc = blake3::hash(b"owner-doc");

        assert_eq!(doc_owner(space, doc), None);
        set_doc_owner(space, doc, peer);
        assert_eq!(doc_owner(space, doc), Some(peer));
        assert!(is_self_doc_owner(space, doc));
    }

    #[test]
    fn ignores_claim_for_other_peer() {
        let me = [1u8; 32];
        let _g = setup(me);
        let space = blake3::hash(b"owner-other-space");
        let doc = blake3::hash(b"owner-other-doc");

        set_doc_owner(space, doc, [2u8; 32]);
        assert_eq!(doc_owner(space, doc), None);
    }
}
