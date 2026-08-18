use std::{
    collections::HashSet,
    sync::Arc,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdDocId,
};

use crate::{
    space::Space,
    tier::Tier,
};

/// A host API surface a document may be granted.
///
/// Every variant has at least one enforcement site; a name with none would be
/// a false statement about what the system protects.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ApiName {
    CreateDocument,
    Event,
    Input,
    InputContext,
    Kv,
    LocalAgent,
    Peer,
    Physics,
    Portal,
    Scene,
    /// Teleporting the local agent into another space.
    Travel,
    Wds,
}

/// Everything the host decides about one document: which tier it came from, and
/// which APIs it may reach.
#[derive(Component, Clone, Debug)]
pub struct DocumentPolicy {
    pub tier:    Tier,
    permissions: Arc<HashSet<ApiName>>,
}

impl Default for DocumentPolicy {
    fn default() -> Self {
        Self::untrusted()
    }
}

impl DocumentPolicy {
    fn new(tier: Tier, names: impl IntoIterator<Item = ApiName>) -> Self {
        Self {
            tier,
            permissions: Arc::new(names.into_iter().collect()),
        }
    }

    #[must_use]
    pub fn untrusted() -> Self {
        Self::new(
            Tier::Untrusted,
            [
                ApiName::Event,
                ApiName::Input,
                ApiName::Kv,
                ApiName::Peer,
                ApiName::Portal,
                ApiName::Scene,
            ],
        )
    }

    #[must_use]
    pub fn space() -> Self {
        Self::new(
            Tier::Space,
            [
                ApiName::CreateDocument,
                ApiName::Event,
                ApiName::Input,
                ApiName::Kv,
                ApiName::LocalAgent,
                ApiName::Peer,
                ApiName::Portal,
                ApiName::Scene,
            ],
        )
    }

    #[must_use]
    pub fn system() -> Self {
        Self::new(
            Tier::System,
            [
                ApiName::CreateDocument,
                ApiName::Event,
                ApiName::Input,
                ApiName::InputContext,
                ApiName::Kv,
                ApiName::LocalAgent,
                ApiName::Peer,
                ApiName::Physics,
                ApiName::Portal,
                ApiName::Scene,
                ApiName::Travel,
                ApiName::Wds,
            ],
        )
    }

    #[must_use]
    pub fn allows(&self, name: ApiName) -> bool {
        self.permissions.contains(&name)
    }

    #[must_use]
    pub fn with(self, name: ApiName) -> Self {
        let mut set = (*self.permissions).clone();
        set.insert(name);
        Self {
            tier:        self.tier,
            permissions: Arc::new(set),
        }
    }
}

pub fn grant_space_permissions(trigger: On<Add, Space>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(DocumentPolicy::space());
}

/// A prefab instance runs with the policy of the document that composed it in.
pub fn inherit_host_permissions(
    trigger: On<Insert, HsdDocId>,
    instances: Query<&ChildOf, (With<Hsd>, Without<DocumentPolicy>)>,
    prims: Query<&HsdChild>,
    hosts: Query<&DocumentPolicy>,
    mut commands: Commands,
) {
    let Ok(prim) = instances.get(trigger.entity).map(ChildOf::parent) else {
        return;
    };
    let Ok(host) = prims.get(prim).map(|c| c.0) else {
        return;
    };
    let Ok(policy) = hosts.get(host) else {
        return;
    };
    commands.entity(trigger.entity).insert(policy.clone());
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
            ApiName::InputContext,
            ApiName::LocalAgent,
            ApiName::Physics,
            ApiName::Travel,
            ApiName::Wds,
        ] {
            assert!(
                !policy.allows(name),
                "a stranger's document must not reach {name:?}"
            );
        }
    }
}
