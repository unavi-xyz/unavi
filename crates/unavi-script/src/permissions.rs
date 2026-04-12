// Scripts are sandboxed by default: only the scene, event, input, and agent
// APIs are available. System scripts (loaded by the client itself, not from
// HSD) receive elevated permissions including WDS access and local-agent
// control. Permissions are checked at linker-build time, so disallowed APIs
// are never wired into the guest's import table.

use std::collections::HashSet;

use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct ScriptPermissions {
    pub api: HashSet<ApiName>,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ApiName {
    Agent,
    CreateDocument,
    Event,
    Input,
    LocalAgent,
    Scene,
    System,
    SystemInput,
    Wds,
}

impl Default for ScriptPermissions {
    fn default() -> Self {
        let mut api = HashSet::default();
        api.insert(ApiName::Agent);
        api.insert(ApiName::Event);
        api.insert(ApiName::Input);
        api.insert(ApiName::Scene);

        Self { api }
    }
}

impl ScriptPermissions {
    #[must_use]
    pub fn system() -> Self {
        let mut perms = Self::default();
        perms.api.insert(ApiName::CreateDocument);
        perms.api.insert(ApiName::LocalAgent);
        perms.api.insert(ApiName::System);
        perms.api.insert(ApiName::SystemInput);
        perms.api.insert(ApiName::Wds);
        perms
    }
}
