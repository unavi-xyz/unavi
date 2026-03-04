use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use blake3::Hash;

#[derive(Component, Clone, Debug)]
pub struct ScriptPermissions {
    pub api: HashSet<ApiName>,
    pub hsd: HashMap<Hash, HashSet<HsdPermissions>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ApiName {
    Agent,
    LocalAgent,
    Scene,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum HsdPermissions {
    Read,
    Write,
}

impl Default for ScriptPermissions {
    fn default() -> Self {
        let mut api = HashSet::default();
        api.insert(ApiName::Agent);
        api.insert(ApiName::Scene);

        Self {
            api,
            hsd: HashMap::default(),
        }
    }
}
