use std::{collections::HashSet, sync::Arc};

use bevy::prelude::*;
use blake3::Hash;

/// Firewall controls how a document may communicate with other documents.
#[derive(Component, Clone, Default, Deref)]
pub struct Firewall(pub Arc<scc::HashMap<Channel, Access>>);

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Channel {
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
        let map = scc::HashMap::new();
        map.insert_sync(Channel::EventRead, Access::Open)
            .expect("insert");
        map.insert_sync(Channel::EventWrite, Access::Open)
            .expect("insert");
        map.insert_sync(Channel::SceneRead, Access::Open)
            .expect("insert");
        Self(Arc::new(map))
    }

    #[must_use]
    pub fn for_child_doc(creator_id: Hash) -> Self {
        let map = scc::HashMap::new();
        map.insert_sync(Channel::EventRead, Access::Open)
            .expect("insert");
        map.insert_sync(Channel::EventWrite, Access::Open)
            .expect("insert");
        map.insert_sync(Channel::SceneRead, Access::Open)
            .expect("insert");
        map.insert_sync(
            Channel::SceneWrite,
            Access::Restricted(HashSet::from([creator_id])),
        )
        .expect("insert");
        Self(Arc::new(map))
    }
}
