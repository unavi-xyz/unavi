use std::{
    collections::HashMap,
    sync::LazyLock,
};

use parking_lot::RwLock;
use xdid::core::did::Did;

use crate::identity;

/// How much a peer is trusted, as one ordinal rung.
///
/// Ordinal rather than a capability matrix because the granularity users
/// actually manage is per-rung: every capability names the minimum rung it
/// needs, and a user moves peers between rungs instead of editing a grid.
///
/// The opinion is the local viewer's and is never gossiped as authoritative,
/// so there is nothing here for a peer or a space to spoof. Ranks *peers*, not
/// documents — [`crate::tier::Tier`] is the document side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Trust {
    /// Ejected. Below the floor every capability sits at, so naming it as a
    /// minimum anywhere would be a mistake.
    Blocked,
    /// Anyone else present. The default, and the rung a normal item must work
    /// at with no configuration and no prompt.
    #[default]
    Guest,
    /// Reachable through someone already trusted.
    Known,
    /// Explicitly trusted.
    Trusted,
    /// The local user.
    Myself,
}

impl Trust {
    /// Whether a peer at this rung clears a capability needing `required`.
    ///
    /// [`Trust::Blocked`] clears nothing, including a requirement of
    /// `Blocked`, so a floor of `Guest` cannot be undercut by naming the
    /// bottom rung.
    #[must_use]
    pub const fn clears(self, required: Self) -> bool {
        !matches!(self, Self::Blocked) && (self as u8) >= (required as u8)
    }
}

/// Rungs the local user has assigned, keyed by DID.
///
/// Keyed to the DID rather than the endpoint because an `EndpointId` rotates,
/// and a table keyed to one would forget every peer on their next device.
static TABLE: LazyLock<RwLock<HashMap<Did, Trust>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// The rung `peer` sits at.
///
/// A peer that has proved no DID cannot rise above [`Trust::Guest`]: an
/// unproven claim is not an identity, so there is nothing to have an opinion
/// about.
#[must_use]
pub fn of_peer(peer: [u8; 32]) -> Trust {
    identity::did_of(peer).map_or(Trust::Guest, |did| of_did(&did))
}

#[must_use]
pub fn of_did(did: &Did) -> Trust {
    TABLE.read().get(did).copied().unwrap_or_default()
}

pub fn set(did: Did, trust: Trust) {
    TABLE.write().insert(did, trust);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn an_unproven_peer_is_a_guest() {
        assert_eq!(of_peer([3; 32]), Trust::Guest);
    }

    #[test]
    fn a_rung_survives_the_endpoint_it_was_learned_on() {
        let did = Did::from_str("did:web:example.com").expect("did");
        set(did.clone(), Trust::Trusted);

        identity::bind([1; 32], did.clone());
        assert_eq!(of_peer([1; 32]), Trust::Trusted);

        identity::unbind([1; 32]);
        identity::bind([2; 32], did);
        assert_eq!(
            of_peer([2; 32]),
            Trust::Trusted,
            "the same DID on a new endpoint keeps its rung"
        );

        identity::unbind([2; 32]);
    }

    #[test]
    fn the_ladder_is_ordered_from_blocked_up() {
        assert!(Trust::Blocked < Trust::Guest);
        assert!(Trust::Guest < Trust::Known);
        assert!(Trust::Known < Trust::Trusted);
        assert!(Trust::Trusted < Trust::Myself);
    }

    #[test]
    fn a_blocked_peer_clears_nothing() {
        for required in [
            Trust::Blocked,
            Trust::Guest,
            Trust::Known,
            Trust::Trusted,
            Trust::Myself,
        ] {
            assert!(
                !Trust::Blocked.clears(required),
                "blocked must not clear {required:?}"
            );
        }
    }

    #[test]
    fn the_default_rung_clears_the_open_default() {
        assert!(Trust::default().clears(Trust::Guest));
        assert!(!Trust::Guest.clears(Trust::Known));
        assert!(Trust::Myself.clears(Trust::Trusted));
    }
}
