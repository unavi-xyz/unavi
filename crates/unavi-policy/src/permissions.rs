use bevy::prelude::*;

use crate::error::PolicyError;

/// A host API surface a document may be granted. Every variant has an
/// enforcement site.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApiName {
    CreateDocument,
    Event,
    /// Reading the local user's own durable identifiers.
    Identity,
    Input,
    InputContext,
    Kv,
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

impl ApiName {
    const fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// The host APIs one document may call.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissions(u16);

impl Default for Permissions {
    fn default() -> Self {
        Self::untrusted()
    }
}

impl Permissions {
    const fn of(names: &[ApiName]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < names.len() {
            bits |= names[i].bit();
            i += 1;
        }
        Self(bits)
    }

    /// The preset for content a peer brought.
    #[must_use]
    pub const fn untrusted() -> Self {
        Self::of(&[
            ApiName::Event,
            ApiName::Input,
            ApiName::Kv,
            ApiName::Peer,
            ApiName::Portal,
            ApiName::Scene,
        ])
    }

    /// The preset for a space's own document.
    #[must_use]
    pub const fn space() -> Self {
        Self::of(&[
            ApiName::CreateDocument,
            ApiName::Event,
            ApiName::Identity,
            ApiName::Input,
            ApiName::Kv,
            ApiName::LocalAgent,
            ApiName::Peer,
            ApiName::Portal,
            ApiName::Scene,
        ])
    }

    /// The preset for the shell and the tools it ships.
    #[must_use]
    pub const fn system() -> Self {
        Self::of(&[
            ApiName::CreateDocument,
            ApiName::Event,
            ApiName::Identity,
            ApiName::Input,
            ApiName::InputContext,
            ApiName::Kv,
            ApiName::LocalAgent,
            ApiName::Peer,
            ApiName::Physics,
            ApiName::Portal,
            ApiName::Scene,
            ApiName::Storage,
            ApiName::Travel,
        ])
    }

    /// Gates a call on this document holding `name`.
    pub const fn require(self, name: ApiName) -> Result<(), PolicyError> {
        if self.0 & name.bit() == 0 {
            Err(PolicyError::Permission(name))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ApiName; 13] = [
        ApiName::CreateDocument,
        ApiName::Event,
        ApiName::Identity,
        ApiName::Input,
        ApiName::InputContext,
        ApiName::Kv,
        ApiName::LocalAgent,
        ApiName::Peer,
        ApiName::Physics,
        ApiName::Portal,
        ApiName::Scene,
        ApiName::Storage,
        ApiName::Travel,
    ];

    const PRIVILEGED: [ApiName; 7] = [
        ApiName::CreateDocument,
        ApiName::Identity,
        ApiName::InputContext,
        ApiName::LocalAgent,
        ApiName::Physics,
        ApiName::Storage,
        ApiName::Travel,
    ];

    #[test]
    fn the_presets_are_nested_by_owner_class() {
        for name in PRIVILEGED {
            assert!(
                Permissions::system().require(name).is_ok(),
                "the system preset must hold {name:?}"
            );
        }
        for name in [
            ApiName::CreateDocument,
            ApiName::Identity,
            ApiName::LocalAgent,
        ] {
            assert!(Permissions::space().require(name).is_ok());
        }
        assert!(Permissions::space().require(ApiName::Travel).is_err());
        assert!(Permissions::space().require(ApiName::Storage).is_err());
    }

    #[test]
    fn untrusted_content_reaches_no_privileged_api() {
        for name in PRIVILEGED {
            assert!(
                Permissions::untrusted().require(name).is_err(),
                "a stranger's document must not reach {name:?}"
            );
        }
    }

    #[test]
    fn a_strangers_document_cannot_read_the_local_users_identifiers() {
        assert!(
            Permissions::untrusted().require(ApiName::Identity).is_err(),
            "a DID is the durable handle the whole trust model is keyed to"
        );
        assert!(Permissions::space().require(ApiName::Identity).is_ok());
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
