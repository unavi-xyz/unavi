use std::collections::HashSet;

use bevy::prelude::*;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct ApiPermissions(HashSet<ApiName>);

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ApiName {
    Agent,
    CreateDocument,
    Event,
    Input,
    InputContext,
    LocalAgent,
    Scene,
    Wds,
}

impl Default for ApiPermissions {
    fn default() -> Self {
        let mut set = HashSet::default();
        set.insert(ApiName::Agent);
        set.insert(ApiName::Event);
        set.insert(ApiName::Input);
        set.insert(ApiName::Scene);
        Self(set)
    }
}

impl ApiPermissions {
    #[must_use]
    pub fn system() -> Self {
        let mut set = Self::default();
        set.insert(ApiName::CreateDocument);
        set.insert(ApiName::InputContext);
        set.insert(ApiName::LocalAgent);
        set.insert(ApiName::Wds);
        set
    }
}
