//! Which host APIs a document may call.

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

/// The host APIs one document may call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissions(u16);

impl Permissions {
    #[must_use]
    pub const fn none() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn of(names: &[HostApi]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < names.len() {
            bits |= names[i].bit();
            i += 1;
        }
        Self(bits)
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
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

    #[test]
    fn a_union_holds_what_either_side_does() {
        let both = Permissions::of(&[HostApi::Event]).union(Permissions::of(&[HostApi::Peer]));
        assert!(both.require(HostApi::Event).is_ok());
        assert!(both.require(HostApi::Peer).is_ok());
        assert!(both.require(HostApi::Scene).is_err());
    }
}
