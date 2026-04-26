use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;

/// Firewall controls how documents may communicate with each other.
#[derive(Component, Clone, Default)]
pub struct Firewall(pub Arc<RwLock<HashMap<XChannel, Access>>>);

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum XChannel {
    EventRead,
    EventWrite,
    SceneRead,
    SceneWrite,
}

#[derive(Clone, Debug)]
pub enum Access {
    Open,
    Restricted(HashSet<Hash>),
}

impl Default for Access {
    fn default() -> Self {
        Self::Restricted(HashSet::default())
    }
}

impl Access {
    #[must_use]
    pub fn permits(&self, id: &Hash) -> bool {
        match self {
            Self::Open => true,
            Self::Restricted(set) => set.contains(id),
        }
    }
}

impl Firewall {
    #[must_use]
    pub fn default_space() -> Self {
        let mut map = HashMap::new();
        map.insert(XChannel::EventRead, Access::Open);
        map.insert(XChannel::EventWrite, Access::Open);
        map.insert(XChannel::SceneRead, Access::Open);
        Self(Arc::new(RwLock::new(map)))
    }

    #[must_use]
    pub fn for_child_doc(creator_id: Hash) -> Self {
        let mut map = HashMap::new();
        map.insert(XChannel::EventRead, Access::Open);
        map.insert(XChannel::EventWrite, Access::Open);
        map.insert(XChannel::SceneRead, Access::Open);
        map.insert(
            XChannel::SceneWrite,
            Access::Restricted(HashSet::from([creator_id])),
        );
        Self(Arc::new(RwLock::new(map)))
    }
}
