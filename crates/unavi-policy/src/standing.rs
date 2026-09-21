use hsd::id::DocId;

use crate::{
    error::PolicyError,
    owner::Owner,
    trust::Trust,
};

/// One document's side of a read or write check.
#[derive(Clone, Copy, Debug)]
pub struct Standing {
    pub owner: Owner,
    pub space: Option<DocId>,
    /// The trust of this document's owner.
    pub trust: Trust,
}

impl Standing {
    /// Whether this document may write `target`.
    pub fn may_write(&self, target: &Self) -> Result<(), PolicyError> {
        if self.same_owner_as(target) {
            return Ok(());
        }
        if !self.co_present_with(target) {
            return Err(PolicyError::NotCoPresent);
        }
        if matches!(self.trust, Trust::Blocked) {
            return Err(PolicyError::Blocked);
        }
        Ok(())
    }

    /// Whether this document may read `target`.
    pub fn may_read(&self, target: &Self) -> Result<(), PolicyError> {
        if self.co_present_with(target) {
            Ok(())
        } else {
            Err(PolicyError::NotCoPresent)
        }
    }

    /// Whether this document may reach anything outside itself.
    pub const fn placed(&self) -> Result<(), PolicyError> {
        if matches!(self.owner, Owner::System) || self.space.is_some() {
            Ok(())
        } else {
            Err(PolicyError::NotCoPresent)
        }
    }

    fn same_owner_as(&self, target: &Self) -> bool {
        match (self.owner, target.owner) {
            (Owner::Peer(a), Owner::Peer(b)) => a == b,
            (Owner::System, Owner::System) => true,
            _ => false,
        }
    }

    /// [`Owner::System`] is co-present with every space.
    fn co_present_with(&self, target: &Self) -> bool {
        matches!(self.owner, Owner::System)
            || matches!((self.space, target.space), (Some(a), Some(b)) if a == b)
    }
}

#[cfg(test)]
mod tests {
    use iroh::{
        EndpointId,
        SecretKey,
    };

    use super::*;

    /// Arbitrary bytes are not a curve point, so the key is derived.
    fn peer(seed: u8) -> EndpointId {
        SecretKey::from_bytes(&[seed; 32]).public()
    }

    fn space(seed: u8) -> DocId {
        DocId([seed; 32])
    }

    fn standing(owner: Owner, space: Option<DocId>, trust: Trust) -> Standing {
        Standing {
            owner,
            space,
            trust,
        }
    }

    #[test]
    fn one_peers_own_documents_reach_each_other_regardless() {
        let mine = peer(1);
        let here = standing(Owner::Peer(mine), Some(space(1)), Trust::Blocked);

        for target_space in [None, Some(space(1)), Some(space(2))] {
            let target = standing(Owner::Peer(mine), target_space, Trust::Guest);
            assert!(
                here.may_write(&target).is_ok(),
                "same-owner must answer before anything else can refuse"
            );
        }
    }

    #[test]
    fn a_stranger_in_the_same_space_may_write_open_content() {
        let here = standing(Owner::Peer(peer(1)), Some(space(1)), Trust::Guest);
        let target = standing(Owner::Peer(peer(2)), Some(space(1)), Trust::Guest);
        assert!(
            here.may_write(&target).is_ok(),
            "a first-time visitor's ball must be kickable with no configuration"
        );
    }

    #[test]
    fn co_presence_is_a_precondition_not_a_trust_level() {
        let here = standing(Owner::Peer(peer(1)), Some(space(1)), Trust::Myself);
        let target = standing(Owner::Peer(peer(2)), Some(space(2)), Trust::Guest);
        assert_eq!(
            here.may_write(&target),
            Err(PolicyError::NotCoPresent),
            "trust cannot substitute for standing in the same space"
        );
    }

    #[test]
    fn every_trust_level_above_blocked_writes_open_content() {
        let target = standing(Owner::Peer(peer(2)), Some(space(1)), Trust::Guest);

        for (trust, allowed) in [
            (Trust::Blocked, false),
            (Trust::Guest, true),
            (Trust::Trusted, true),
            (Trust::Myself, true),
        ] {
            let caller = standing(Owner::Peer(peer(1)), Some(space(1)), trust);
            assert_eq!(caller.may_write(&target).is_ok(), allowed, "{trust:?}");
        }
    }

    #[test]
    fn a_blocked_peer_is_refused_content_it_does_not_own() {
        let here = standing(Owner::Peer(peer(1)), Some(space(1)), Trust::Blocked);
        let target = standing(Owner::Peer(peer(2)), Some(space(1)), Trust::Guest);
        assert_eq!(here.may_write(&target), Err(PolicyError::Blocked));
    }

    #[test]
    fn content_a_space_holds_is_not_one_owner() {
        let first = standing(Owner::Space(space(1)), Some(space(1)), Trust::Guest);
        let second = standing(Owner::Space(space(1)), Some(space(1)), Trust::Blocked);
        assert!(
            !first.same_owner_as(&second),
            "the space owning both must not waive the check for either"
        );
        assert_eq!(
            second.may_write(&first),
            Err(PolicyError::Blocked),
            "space-owned content still faces co-presence and the block list"
        );
    }

    #[test]
    fn two_unknowns_are_not_one_peer() {
        let anon = standing(Owner::Space(space(1)), Some(space(1)), Trust::Guest);
        let other_anon = standing(Owner::Space(space(1)), Some(space(1)), Trust::Guest);
        assert!(!anon.same_owner_as(&other_anon));

        let owned = standing(Owner::Peer(peer(1)), Some(space(1)), Trust::Guest);
        assert!(!anon.same_owner_as(&owned));
        assert!(!owned.same_owner_as(&anon));
        assert!(owned.same_owner_as(&owned));
        assert!(!owned.same_owner_as(&standing(
            Owner::Peer(peer(2)),
            Some(space(1)),
            Trust::Guest
        )));

        let shell = standing(Owner::System, None, Trust::Myself);
        assert!(
            shell.same_owner_as(&shell),
            "system documents are one owner"
        );
        assert!(!anon.same_owner_as(&shell));
        assert!(!owned.same_owner_as(&shell));
    }

    #[test]
    fn the_system_owner_crosses_a_space_boundary_in_one_direction() {
        let shell = standing(Owner::System, None, Trust::Myself);
        let prop = standing(Owner::Peer(peer(2)), Some(space(1)), Trust::Guest);

        assert!(shell.may_write(&prop).is_ok());
        assert!(shell.placed().is_ok(), "the shell is placed by its owner");
        assert_eq!(
            prop.may_write(&shell),
            Err(PolicyError::NotCoPresent),
            "content a peer brought must not write the shell it stands in"
        );
    }

    #[test]
    fn an_unplaced_document_reaches_nothing_it_does_not_own() {
        let orphan = standing(Owner::Peer(peer(1)), None, Trust::Guest);
        assert_eq!(orphan.placed(), Err(PolicyError::NotCoPresent));
        assert!(
            standing(Owner::Peer(peer(1)), Some(space(1)), Trust::Guest)
                .placed()
                .is_ok()
        );
    }

    #[test]
    fn reads_are_open_within_a_space_and_closed_across_one() {
        let here = standing(Owner::Peer(peer(1)), Some(space(1)), Trust::Guest);
        let beside = standing(Owner::Peer(peer(2)), Some(space(1)), Trust::Guest);
        let elsewhere = standing(Owner::Peer(peer(2)), Some(space(2)), Trust::Guest);

        assert!(here.may_read(&beside).is_ok());
        assert_eq!(
            here.may_read(&elsewhere),
            Err(PolicyError::NotCoPresent),
            "a namespace the reader has no id for stays unreadable"
        );
    }
}
