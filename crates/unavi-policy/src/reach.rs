use bevy::prelude::*;
use hsd::id::DocId;
use iroh::EndpointId;

use crate::{
    error::PolicyError,
    tier::Tier,
    trust::Trust,
};

/// What a document demands of whoever writes to it.
///
/// One rung, not a per-channel allow-list over documents. Reads carry no
/// setting at all — within a space they are open, and space membership is the
/// gate.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reach {
    pub writes_from: Trust,
}

impl Default for Reach {
    /// Open; content that wants to be tamper-proof raises the rung.
    fn default() -> Self {
        Self {
            writes_from: Trust::Guest,
        }
    }
}

impl Reach {
    /// Admits only the local user's own documents. Not a seal against
    /// yourself — same-owner writes answer before the rung is consulted — it
    /// refuses every *other* peer.
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

/// Where a document stands: where it was loaded from, what it demands of
/// writers, which space it is in, and who owns it at what rung.
///
/// Resolving one needs the replicated pins, which live above this crate. The
/// rules below only need the answers, so they take a pair of these.
#[derive(Clone, Copy, Debug)]
pub struct Standing {
    pub tier:  Tier,
    pub reach: Reach,
    pub space: Option<DocId>,
    /// The peer whose pin owns this document, absent when nothing attributes
    /// it.
    pub owner: Option<EndpointId>,
    /// The rung `owner` sits at, or [`Trust::Guest`] when there is no owner.
    ///
    /// A document nobody can be shown to have authored is judged as a
    /// stranger's, never as the local user's. The fail-open direction here is
    /// what decides whether unattributable content can write everything you
    /// own.
    pub trust: Trust,
}

impl Standing {
    /// Whether a document standing here may write to one standing at `target`.
    ///
    /// The ordering is the whole policy: same-owner answers unconditionally,
    /// co-presence is then a precondition, and the rung is the decision.
    pub fn may_write(&self, target: &Self) -> Result<(), PolicyError> {
        if self.same_owner_as(target) {
            return Ok(());
        }
        if !self.co_present_with(target) {
            return Err(PolicyError::NotCoPresent);
        }
        if target.reach.admits(self.trust) {
            Ok(())
        } else {
            Err(PolicyError::Rung {
                required: target.reach.writes_from,
                actual:   self.trust,
            })
        }
    }

    /// Whether a document standing here may read one standing at `target`.
    ///
    /// Reads are open within a space, so membership is the whole gate. A
    /// document that wants to be unreadable has to live in a namespace the
    /// reader has no id for.
    pub fn may_read(&self, target: &Self) -> Result<(), PolicyError> {
        if self.co_present_with(target) {
            Ok(())
        } else {
            Err(PolicyError::NotCoPresent)
        }
    }

    /// Whether this document is placed well enough to reach anything outside
    /// itself.
    ///
    /// A document outside every space has no co-presence to appeal to, so its
    /// only possible reach is same-owner. Refusing up front stops
    /// unattributable content riding an ownership answer it did not earn.
    pub const fn placed(&self) -> Result<(), PolicyError> {
        if self.tier.crosses_space_boundaries() || self.space.is_some() {
            Ok(())
        } else {
            Err(PolicyError::NotCoPresent)
        }
    }

    /// An unknown owner on either side is never the same owner. Answering
    /// "yes" for a pair the host cannot attribute is what let a peer's prefab
    /// instance — which is never pinned and so has no owner of its own — read
    /// as locally authored and write anything the local user owns.
    fn same_owner_as(&self, target: &Self) -> bool {
        matches!((self.owner, target.owner), (Some(a), Some(b)) if a == b)
    }

    /// The shell is placed outside any space and still has to reach into
    /// whichever one the user is standing in, so its tier answers before the
    /// spaces are compared.
    fn co_present_with(&self, target: &Self) -> bool {
        self.tier.crosses_space_boundaries()
            || matches!((self.space, target.space), (Some(a), Some(b)) if a == b)
    }
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;

    use super::*;

    /// A distinct, valid endpoint id per seed. Arbitrary bytes are not a curve
    /// point, so a key has to be derived rather than written down.
    fn peer(seed: u8) -> EndpointId {
        SecretKey::from_bytes(&[seed; 32]).public()
    }

    fn space(seed: u8) -> DocId {
        DocId([seed; 32])
    }

    /// Untrusted content owned by `owner`, standing in `space`, open to anyone.
    fn standing(owner: Option<EndpointId>, space: Option<DocId>, trust: Trust) -> Standing {
        Standing {
            tier: Tier::Untrusted,
            reach: Reach::default(),
            space,
            owner,
            trust,
        }
    }

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
        let mine = peer(1);
        let here = standing(Some(mine), Some(space(1)), Trust::Guest);

        for (reach, target_space) in [
            (Reach::own_only(), None),
            (Reach::own_only(), Some(space(1))),
            (Reach::default(), Some(space(2))),
        ] {
            let target = Standing {
                reach,
                space: target_space,
                ..standing(Some(mine), target_space, Trust::Guest)
            };
            assert!(
                here.may_write(&target).is_ok(),
                "same-owner must answer before anything else can refuse"
            );
        }
    }

    #[test]
    fn a_stranger_in_the_same_space_may_write_open_content() {
        let here = standing(Some(peer(1)), Some(space(1)), Trust::Guest);
        let target = standing(Some(peer(2)), Some(space(1)), Trust::Guest);
        assert!(
            here.may_write(&target).is_ok(),
            "a first-time visitor's ball must be kickable with no configuration"
        );
    }

    #[test]
    fn co_presence_is_a_precondition_not_a_rung() {
        let here = standing(Some(peer(1)), Some(space(1)), Trust::Myself);
        let target = standing(Some(peer(2)), Some(space(2)), Trust::Guest);
        assert_eq!(
            here.may_write(&target),
            Err(PolicyError::NotCoPresent),
            "trust cannot substitute for standing in the same space"
        );
    }

    #[test]
    fn raising_the_rung_shuts_out_the_rungs_below_it() {
        let guarded = Standing {
            reach: Reach {
                writes_from: Trust::Trusted,
            },
            ..standing(Some(peer(2)), Some(space(1)), Trust::Guest)
        };

        for (rung, allowed) in [(Trust::Guest, false), (Trust::Trusted, true)] {
            let caller = standing(Some(peer(1)), Some(space(1)), rung);
            assert_eq!(caller.may_write(&guarded).is_ok(), allowed, "{rung:?}");
        }
    }

    #[test]
    fn a_blocked_peer_is_refused_content_it_does_not_own() {
        let here = standing(Some(peer(1)), Some(space(1)), Trust::Blocked);
        let target = standing(Some(peer(2)), Some(space(1)), Trust::Guest);
        assert!(here.may_write(&target).is_err());
    }

    #[test]
    fn an_unattributable_document_is_nobodys_own() {
        let anon = standing(None, Some(space(1)), Trust::Guest);
        let other_anon = standing(None, Some(space(1)), Trust::Guest);
        assert!(
            !anon.same_owner_as(&other_anon),
            "two unknowns are not one peer"
        );

        let owned = standing(Some(peer(1)), Some(space(1)), Trust::Guest);
        assert!(!anon.same_owner_as(&owned));
        assert!(!owned.same_owner_as(&anon));
        assert!(owned.same_owner_as(&owned));
        assert!(!owned.same_owner_as(&standing(Some(peer(2)), Some(space(1)), Trust::Guest)));
    }

    /// The shell stands outside every space and still has to reach into the one
    /// the user occupies.
    #[test]
    fn the_system_tier_crosses_a_space_boundary_in_one_direction() {
        let shell = Standing {
            tier: Tier::System,
            ..standing(Some(peer(1)), None, Trust::Myself)
        };
        let prop = standing(Some(peer(2)), Some(space(1)), Trust::Guest);

        assert!(shell.may_write(&prop).is_ok());
        assert!(shell.placed().is_ok(), "the shell is placed by its tier");
        assert_eq!(
            prop.may_write(&shell),
            Err(PolicyError::NotCoPresent),
            "content a peer brought must not write the shell it stands in"
        );
    }

    #[test]
    fn an_unplaced_document_reaches_nothing_it_does_not_own() {
        let orphan = standing(Some(peer(1)), None, Trust::Guest);
        assert_eq!(orphan.placed(), Err(PolicyError::NotCoPresent));
        assert!(
            standing(Some(peer(1)), Some(space(1)), Trust::Guest)
                .placed()
                .is_ok()
        );
    }

    #[test]
    fn reads_are_open_within_a_space_and_closed_across_one() {
        let here = standing(Some(peer(1)), Some(space(1)), Trust::Guest);
        let beside = standing(Some(peer(2)), Some(space(1)), Trust::Guest);
        let elsewhere = standing(Some(peer(2)), Some(space(2)), Trust::Guest);

        assert!(here.may_read(&beside).is_ok());
        assert_eq!(
            here.may_read(&elsewhere),
            Err(PolicyError::NotCoPresent),
            "a namespace the reader has no id for stays unreadable"
        );
    }
}
