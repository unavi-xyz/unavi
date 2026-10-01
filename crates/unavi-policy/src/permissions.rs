//! Which host APIs a document may call, by how far its author is trusted.

use crate::trust::Trust;

/// A call refused because the document lacks the API it needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("permission denied: {0:?}")]
pub struct PermissionDenied(pub HostApi);

/// A host API surface a document may be granted. Every variant has an
/// enforcement site.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostApi {
    Commit,
    CreateDocument,
    Event,
    /// Reading the local user's own durable identifiers.
    Identity,
    Input,
    InputContext,
    LocalAgent,
    Peer,
    Physics,
    Portal,
    Scene,
    /// Reading the documents this node holds, including its root document and
    /// the registry views it follows.
    Storage,
    /// Teleporting the local agent into another space.
    Travel,
}

impl HostApi {
    const fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// The host APIs one document may call, derived from how far its author is
/// trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissions(u16);

impl Permissions {
    const fn of(names: &[HostApi]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < names.len() {
            bits |= names[i].bit();
            i += 1;
        }
        Self(bits)
    }

    /// The set a document authored by someone at `trust` may call.
    ///
    /// The rungs gate only what is uncommon and consequential, so a first-time
    /// visitor's ball, door or whiteboard works with no configuration. Reading
    /// the scene, speaking on channels, spawning documents and opening portals
    /// are all quota-bounded rather than trust-gated, which is why they sit at
    /// the floor.
    #[must_use]
    pub const fn for_trust(trust: Trust) -> Self {
        match trust {
            // A blocked author reaches nothing. The connection layer refuses
            // the peer first; this is what remains if content of theirs is
            // already resident.
            Trust::Blocked => Self::of(&[]),
            // An unproven author can make its own prop work, but cannot
            // multiply itself or send anyone elsewhere.
            Trust::Anonymous => Self::of(&[
                HostApi::Event,
                HostApi::Input,
                HostApi::Peer,
                HostApi::Scene,
            ]),
            Trust::Guest => Self(
                Self::for_trust(Trust::Anonymous).0
                    | Self::of(&[HostApi::CreateDocument, HostApi::Portal]).0,
            ),
            // Identity is the durable handle the whole trust model is keyed
            // to, and the agent pose is continuous motion capture of a real
            // person. Neither is something a stranger's prop may read.
            // Commit is what makes an opinion durable, so a malicious prop
            // standing in its owner's room must not hold it — possession
            // already keeps it from another peer's document, and trust is
            // what stops it writing the room it stands in. The shell and the
            // tools it ships are authored at this rung; content is not.
            Trust::Trusted => Self(
                Self::for_trust(Trust::Guest).0
                    | Self::of(&[HostApi::Commit, HostApi::Identity, HostApi::LocalAgent]).0,
            ),
            // Global input listening, the physics solver and cross-space
            // reach. Nothing the local user did not author holds these.
            Trust::Myself => Self(
                Self::for_trust(Trust::Trusted).0
                    | Self::of(&[
                        HostApi::InputContext,
                        HostApi::Physics,
                        HostApi::Storage,
                        HostApi::Travel,
                    ])
                    .0,
            ),
        }
    }

    /// Gates a call on this document holding `name`.
    pub const fn require(self, name: HostApi) -> Result<(), PermissionDenied> {
        if self.0 & name.bit() == 0 {
            Err(PermissionDenied(name))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [HostApi; 13] = [
        HostApi::Commit,
        HostApi::CreateDocument,
        HostApi::Event,
        HostApi::Identity,
        HostApi::Input,
        HostApi::InputContext,
        HostApi::LocalAgent,
        HostApi::Peer,
        HostApi::Physics,
        HostApi::Portal,
        HostApi::Scene,
        HostApi::Storage,
        HostApi::Travel,
    ];

    const LADDER: [Trust; 5] = [
        Trust::Blocked,
        Trust::Anonymous,
        Trust::Guest,
        Trust::Trusted,
        Trust::Myself,
    ];

    #[test]
    fn a_rung_holds_everything_the_rung_below_it_does() {
        for pair in LADDER.windows(2) {
            let lower = Permissions::for_trust(pair[0]);
            let upper = Permissions::for_trust(pair[1]);
            assert_eq!(
                lower.0 & upper.0,
                lower.0,
                "{:?} must not drop anything {:?} holds",
                pair[1],
                pair[0]
            );
        }
    }

    #[test]
    fn a_blocked_author_reaches_nothing() {
        for name in ALL {
            assert!(
                Permissions::for_trust(Trust::Blocked)
                    .require(name)
                    .is_err()
            );
        }
    }

    #[test]
    fn an_unproven_author_cannot_spawn_or_send_anyone_away() {
        let anonymous = Permissions::for_trust(Trust::Anonymous);
        assert!(anonymous.require(HostApi::CreateDocument).is_err());
        assert!(anonymous.require(HostApi::Portal).is_err());
        assert!(anonymous.require(HostApi::Scene).is_ok());
    }

    #[test]
    fn a_first_time_visitors_prop_works_with_no_configuration() {
        let guest = Permissions::for_trust(Trust::Guest);
        for name in [
            HostApi::CreateDocument,
            HostApi::Event,
            HostApi::Input,
            HostApi::Peer,
            HostApi::Portal,
            HostApi::Scene,
        ] {
            assert!(guest.require(name).is_ok(), "a guest needs {name:?}");
        }
    }

    #[test]
    fn a_strangers_document_cannot_read_the_local_user() {
        let guest = Permissions::for_trust(Trust::Guest);
        assert!(
            guest.require(HostApi::Identity).is_err(),
            "a DID is the durable handle the whole trust model is keyed to"
        );
        assert!(
            guest.require(HostApi::LocalAgent).is_err(),
            "the agent pose is continuous motion capture of a real person"
        );
    }

    #[test]
    fn only_the_local_users_own_content_reaches_the_shell_apis() {
        for name in [
            HostApi::InputContext,
            HostApi::Physics,
            HostApi::Storage,
            HostApi::Travel,
        ] {
            assert!(
                Permissions::for_trust(Trust::Trusted)
                    .require(name)
                    .is_err()
            );
            assert!(Permissions::for_trust(Trust::Myself).require(name).is_ok());
        }
    }

    /// Every name has to fit the bitfield, and no two may share a bit.
    #[test]
    fn each_api_name_occupies_its_own_bit() {
        let mut seen = 0_u16;
        for name in ALL {
            assert_eq!(seen & name.bit(), 0, "{name:?} shares a bit");
            seen |= name.bit();
        }
        assert_eq!(Permissions::of(&ALL).0, seen);
    }
}
