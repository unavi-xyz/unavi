use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
};

use bevy::prelude::*;
use blake3::Hash;
use parking_lot::RwLock;

/// Firewall controls how a document may communicate with other documents.
#[derive(Component, Clone, Deref)]
pub struct Firewall(pub Arc<RwLock<HashMap<Channel, Access>>>);

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Channel {
    EventRead,
    EventWrite,
    KvRead,
    KvWrite,
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
    pub fn for_child_doc(creator_id: Hash) -> Self {
        let mut map = HashMap::new();
        map.insert(Channel::EventRead, Access::Open);
        map.insert(Channel::EventWrite, Access::Open);
        map.insert(Channel::KvRead, Access::Open);
        map.insert(
            Channel::KvWrite,
            Access::Restricted(HashSet::from([creator_id])),
        );
        map.insert(Channel::SceneRead, Access::Open);
        map.insert(
            Channel::SceneWrite,
            Access::Restricted(HashSet::from([creator_id])),
        );
        Self(Arc::new(RwLock::new(map)))
    }
}

impl Default for Firewall {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(Channel::EventRead, Access::Open);
        map.insert(Channel::EventWrite, Access::Open);
        map.insert(Channel::KvRead, Access::Open);
        map.insert(Channel::SceneRead, Access::Open);
        Self(Arc::new(RwLock::new(map)))
    }
}
