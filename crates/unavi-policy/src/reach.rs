use std::{
    collections::HashMap,
    sync::LazyLock,
};

use bevy::prelude::*;
use bevy_hsd::HsdDocId;
use hsd::id::DocId;
use parking_lot::RwLock;

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
    /// Refuses every peer, including the local user, so nothing script-side
    /// can write the document.
    #[must_use]
    pub const fn sealed() -> Self {
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
pub fn permits(
    trust: Trust,
    target: Reach,
    same_owner: bool,
    co_present: bool,
) -> Result<(), PolicyError> {
    if same_owner {
        return Ok(());
    }
    if !co_present {
        return Err(PolicyError::Reach(
            "documents are not in the same space".into(),
        ));
    }
    if target.admits(trust) {
        Ok(())
    } else {
        Err(PolicyError::Reach(format!(
            "writes need {:?}, caller is {trust:?}",
            target.writes_from
        )))
    }
}

static REACH_REGISTRY: LazyLock<RwLock<HashMap<DocId, Reach>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// What `doc` demands of its writers.
///
/// An unregistered document answers [`Reach::default`]. That is the stated
/// default rather than a fallback: a document says nothing until it wants to be
/// harder to write than open, so silence and openness are the same statement.
#[must_use]
pub fn required(doc: DocId) -> Reach {
    REACH_REGISTRY.read().get(&doc).copied().unwrap_or_default()
}

pub fn set_required(doc: DocId, reach: Reach) {
    REACH_REGISTRY.write().insert(doc, reach);
}

pub fn register_reach(trigger: On<Add, Reach>, docs: Query<(&HsdDocId, &Reach)>) {
    let Ok((doc, reach)) = docs.get(trigger.entity) else {
        return;
    };
    set_required(doc.0, *reach);
}

pub fn deregister_reach(trigger: On<Remove, HsdDocId>, docs: Query<&HsdDocId>) {
    let Ok(doc) = docs.get(trigger.entity) else {
        return;
    };
    REACH_REGISTRY.write().remove(&doc.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unstated_reach_admits_a_guest() {
        assert!(required(DocId([9; 32])).admits(Trust::Guest));
    }

    #[test]
    fn a_sealed_document_refuses_everyone_below_the_local_user() {
        let sealed = Reach::sealed();
        assert!(!sealed.admits(Trust::Guest));
        assert!(!sealed.admits(Trust::Trusted));
        assert!(sealed.admits(Trust::Myself));
    }

    #[test]
    fn a_blocked_peer_is_refused_by_an_open_document() {
        assert!(!Reach::default().admits(Trust::Blocked));
    }

    #[test]
    fn one_peers_own_documents_reach_each_other_regardless() {
        for (target, co_present) in [
            (Reach::sealed(), false),
            (Reach::sealed(), true),
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
}
