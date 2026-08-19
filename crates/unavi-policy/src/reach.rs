use bevy::prelude::*;

use crate::{
    error::PolicyError,
    trust::Trust,
};

/// What a document demands of whoever writes to it.
///
/// One rung, not a per-channel allow-list over documents: an N×N matrix over
/// documents is something nothing can populate sensibly, which is why the
/// firewall it replaces had exactly two states in practice. Reads carry no
/// setting at all — within a space they are open, and space membership is the
/// gate.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reach {
    pub writes_from: Trust,
}

impl Default for Reach {
    /// Open. A shared world where a stranger's ball cannot be kicked is the
    /// wrong default; content that wants to be tamper-proof raises the rung.
    fn default() -> Self {
        Self {
            writes_from: Trust::Guest,
        }
    }
}

impl Reach {
    /// Admits only the local user's own documents.
    ///
    /// Same-owner writes answer before the rung is consulted, so this is not a
    /// seal against yourself: it refuses every *other* peer, which is what the
    /// shell needs so a stranger cannot write its scene or speak on its
    /// channels.
    #[must_use]
    pub const fn own_only() -> Self {
        Self {
            writes_from: Trust::Myself,
        }
    }

    #[must_use]
    pub const fn admits(self, trust: Trust) -> bool {
        trust.clears(self.writes_from)
    }
}

/// Whether a write is in reach, given who is asking and where both documents
/// stand.
///
/// The ordering is the whole policy. Same-owner comes first and answers
/// unconditionally, because it is one peer's content on both sides and a
/// boundary there protects nobody. Everything after it is the cross-owner case,
/// where co-presence is a precondition and the rung is the decision.
pub const fn permits(
    trust: Trust,
    target: Reach,
    same_owner: bool,
    co_present: bool,
) -> Result<(), PolicyError> {
    if same_owner {
        return Ok(());
    }
    if !co_present {
        return Err(PolicyError::NotCoPresent);
    }
    if target.admits(trust) {
        Ok(())
    } else {
        Err(PolicyError::Rung {
            required: target.writes_from,
            actual:   trust,
        })
    }
}

/// Whether two documents are owned by the same peer.
///
/// An unknown owner on either side is never the same owner. Answering "yes"
/// for a pair the host cannot attribute is what let a peer's prefab instance —
/// which is never pinned and so has no owner of its own — read as locally
/// authored and write anything the local user owns.
#[must_use]
pub const fn same_owner(a: Option<[u8; 32]>, b: Option<[u8; 32]>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => const_eq(&a, &b),
        _ => false,
    }
}

const fn const_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut i = 0;
    while i < 32 {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unstated_reach_admits_a_guest() {
        assert!(Reach::default().admits(Trust::Guest));
    }

    #[test]
    fn an_own_only_document_refuses_everyone_below_the_local_user() {
        let own_only = Reach::own_only();
        assert!(!own_only.admits(Trust::Guest));
        assert!(!own_only.admits(Trust::Trusted));
        assert!(own_only.admits(Trust::Myself));
    }

    #[test]
    fn a_blocked_peer_is_refused_by_an_open_document() {
        assert!(!Reach::default().admits(Trust::Blocked));
    }

    #[test]
    fn one_peers_own_documents_reach_each_other_regardless() {
        for (target, co_present) in [
            (Reach::own_only(), false),
            (Reach::own_only(), true),
            (Reach::default(), false),
        ] {
            assert!(
                permits(Trust::Guest, target, true, co_present).is_ok(),
                "same-owner must answer before anything else can refuse"
            );
        }
    }

    #[test]
    fn a_stranger_in_the_same_space_may_write_open_content() {
        assert!(
            permits(Trust::Guest, Reach::default(), false, true).is_ok(),
            "a first-time visitor's ball must be kickable with no configuration"
        );
    }

    #[test]
    fn co_presence_is_a_precondition_not_a_rung() {
        assert!(
            permits(Trust::Myself, Reach::default(), false, false).is_err(),
            "trust cannot substitute for standing in the same space"
        );
    }

    #[test]
    fn raising_the_rung_shuts_out_the_rungs_below_it() {
        let guarded = Reach {
            writes_from: Trust::Trusted,
        };
        assert!(permits(Trust::Guest, guarded, false, true).is_err());
        assert!(permits(Trust::Known, guarded, false, true).is_err());
        assert!(permits(Trust::Trusted, guarded, false, true).is_ok());
    }

    #[test]
    fn a_blocked_peer_is_refused_content_it_does_not_own() {
        assert!(permits(Trust::Blocked, Reach::default(), false, true).is_err());
    }

    #[test]
    fn an_unattributable_document_is_nobodys_own() {
        assert!(!same_owner(None, None), "two unknowns are not one peer");
        assert!(!same_owner(Some([1; 32]), None));
        assert!(!same_owner(None, Some([1; 32])));
        assert!(same_owner(Some([1; 32]), Some([1; 32])));
        assert!(!same_owner(Some([1; 32]), Some([2; 32])));
    }
}
