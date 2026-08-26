use bevy::prelude::*;

use crate::tier::Tier;

/// A host API surface a document may be granted.
///
/// Every variant has at least one enforcement site; a name with none would be
/// a false statement about what the system protects.
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
    /// Reading the docs this node holds: its own root doc and the registry
    /// views it follows.
    Storage,
    /// Teleporting the local agent into another space.
    Travel,
}

impl ApiName {
    const fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// The set of APIs a document holds, as one word.
///
/// [`DocumentPolicy::allows`] answers every host call, so the check is an
/// `and` on a bitfield and the whole policy stays `Copy`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ApiSet(u16);

impl ApiSet {
    #[must_use]
    pub const fn none() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn with(self, name: ApiName) -> Self {
        Self(self.0 | name.bit())
    }

    #[must_use]
    pub const fn contains(self, name: ApiName) -> bool {
        self.0 & name.bit() != 0
    }
}

/// Everything the host decides about one document: which tier it came from, and
/// which APIs it may reach.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentPolicy {
    pub tier:    Tier,
    permissions: ApiSet,
}

impl Default for DocumentPolicy {
    fn default() -> Self {
        Self::untrusted()
    }
}

impl DocumentPolicy {
    const fn new(tier: Tier, permissions: ApiSet) -> Self {
        Self { tier, permissions }
    }

    #[must_use]
    pub const fn untrusted() -> Self {
        Self::new(
            Tier::Untrusted,
            ApiSet::none()
                .with(ApiName::Event)
                .with(ApiName::Input)
                .with(ApiName::Kv)
                .with(ApiName::Peer)
                .with(ApiName::Portal)
                .with(ApiName::Scene),
        )
    }

    #[must_use]
    pub const fn space() -> Self {
        Self::new(
            Tier::Space,
            ApiSet::none()
                .with(ApiName::CreateDocument)
                .with(ApiName::Event)
                .with(ApiName::Identity)
                .with(ApiName::Input)
                .with(ApiName::Kv)
                .with(ApiName::LocalAgent)
                .with(ApiName::Peer)
                .with(ApiName::Portal)
                .with(ApiName::Scene),
        )
    }

    #[must_use]
    pub const fn system() -> Self {
        Self::new(
            Tier::System,
            ApiSet::none()
                .with(ApiName::CreateDocument)
                .with(ApiName::Event)
                .with(ApiName::Identity)
                .with(ApiName::Input)
                .with(ApiName::InputContext)
                .with(ApiName::Kv)
                .with(ApiName::LocalAgent)
                .with(ApiName::Peer)
                .with(ApiName::Physics)
                .with(ApiName::Portal)
                .with(ApiName::Scene)
                .with(ApiName::Travel)
                .with(ApiName::Storage),
        )
    }

    #[must_use]
    pub const fn allows(self, name: ApiName) -> bool {
        self.permissions.contains(name)
    }

    /// Gates a call on this document holding `name`.
    pub const fn require(self, name: ApiName) -> Result<(), crate::error::PolicyError> {
        if self.allows(name) {
            Ok(())
        } else {
            Err(crate::error::PolicyError::Permission(name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_system_tier_crosses_space_boundaries() {
        assert!(DocumentPolicy::system().tier.crosses_space_boundaries());
        assert!(!DocumentPolicy::space().tier.crosses_space_boundaries());
        assert!(!DocumentPolicy::untrusted().tier.crosses_space_boundaries());
    }

    #[test]
    fn untrusted_content_reaches_no_privileged_api() {
        let policy = DocumentPolicy::untrusted();
        for name in [
            ApiName::CreateDocument,
            ApiName::Identity,
            ApiName::InputContext,
            ApiName::LocalAgent,
            ApiName::Physics,
            ApiName::Travel,
            ApiName::Storage,
        ] {
            assert!(
                !policy.allows(name),
                "a stranger's document must not reach {name:?}"
            );
        }
    }

    #[test]
    fn a_strangers_document_cannot_read_the_local_users_identifiers() {
        assert!(
            DocumentPolicy::untrusted()
                .require(ApiName::Identity)
                .is_err(),
            "a DID is the durable handle the whole trust model is keyed to"
        );
        assert!(DocumentPolicy::space().require(ApiName::Identity).is_ok());
    }

    /// Every name has to fit the bitfield, and no two may share a bit.
    #[test]
    fn each_api_name_occupies_its_own_bit() {
        let all = [
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
            ApiName::Travel,
            ApiName::Storage,
        ];
        let mut seen = ApiSet::none();
        for name in all {
            assert!(!seen.contains(name), "{name:?} shares a bit");
            seen = seen.with(name);
        }
        for name in all {
            assert!(seen.contains(name));
        }
    }
}
