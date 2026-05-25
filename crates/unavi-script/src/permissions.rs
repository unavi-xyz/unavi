use std::{
    collections::HashSet,
    sync::Arc,
};

use bevy::prelude::*;

#[derive(Component, Clone, Debug, Deref)]
pub struct ApiPermissions(Arc<HashSet<ApiName>>);

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
    pub fn system() -> Self {
        let mut set = HashSet::default();
        set.insert(ApiName::Agent);
        set.insert(ApiName::LocalAgent);
        set.insert(ApiName::CreateDocument);
        set.insert(ApiName::Event);
        set.insert(ApiName::Input);
        set.insert(ApiName::InputContext);
        set.insert(ApiName::Scene);
        set.insert(ApiName::Wds);
        Self(Arc::new(set))
    }
}
