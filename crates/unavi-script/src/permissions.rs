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
use unavi_space::Space;

#[derive(Component, Clone, Debug, Deref)]
pub struct ApiPermissions(Arc<HashSet<ApiName>>);

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ApiName {
    Agent,
    CreateDocument,
    Event,
    Input,
    InputContext,
    Kv,
    LocalAgent,
    Peer,
    Physics,
    Portal,
    Registry,
    Scene,
    System,
    Wds,
}

impl Default for ApiPermissions {
    fn default() -> Self {
        let mut set = HashSet::default();
        set.insert(ApiName::Agent);
        set.insert(ApiName::Event);
        set.insert(ApiName::Input);
        set.insert(ApiName::Kv);
        set.insert(ApiName::Peer);
        set.insert(ApiName::Portal);
        set.insert(ApiName::Scene);
        Self(Arc::new(set))
    }
}

impl ApiPermissions {
    #[must_use]
    pub fn with(self, name: ApiName) -> Self {
        let mut set = (*self.0).clone();
        set.insert(name);
        Self(Arc::new(set))
    }

    #[must_use]
    pub fn space() -> Self {
        let mut set = HashSet::default();
        set.insert(ApiName::Agent);
        set.insert(ApiName::CreateDocument);
        set.insert(ApiName::Event);
        set.insert(ApiName::Input);
        set.insert(ApiName::Kv);
        set.insert(ApiName::LocalAgent);
        set.insert(ApiName::Peer);
        set.insert(ApiName::Portal);
        set.insert(ApiName::Scene);
        Self(Arc::new(set))
    }

    #[must_use]
    pub fn system() -> Self {
        let mut set = HashSet::default();
        set.insert(ApiName::Agent);
        set.insert(ApiName::LocalAgent);
        set.insert(ApiName::CreateDocument);
        set.insert(ApiName::Event);
        set.insert(ApiName::Input);
        set.insert(ApiName::InputContext);
        set.insert(ApiName::Kv);
        set.insert(ApiName::Peer);
        set.insert(ApiName::Physics);
        set.insert(ApiName::Portal);
        set.insert(ApiName::Registry);
        set.insert(ApiName::Scene);
        set.insert(ApiName::System);
        set.insert(ApiName::Wds);
        Self(Arc::new(set))
    }
}

pub fn grant_space_permissions(trigger: On<Add, Space>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(ApiPermissions::space());
}

/// A prefab instance runs with the permissions of the document that composed
/// it in.
pub fn inherit_host_permissions(
    trigger: On<Insert, HsdDocId>,
    instances: Query<&ChildOf, (With<Hsd>, Without<ApiPermissions>)>,
    prims: Query<&HsdChild>,
    hosts: Query<&ApiPermissions>,
    mut commands: Commands,
) {
    let Ok(prim) = instances.get(trigger.entity).map(ChildOf::parent) else {
        return;
    };
    let Ok(host) = prims.get(prim).map(|c| c.0) else {
        return;
    };
    let Ok(permissions) = hosts.get(host) else {
        return;
    };
    commands.entity(trigger.entity).insert(permissions.clone());
}
