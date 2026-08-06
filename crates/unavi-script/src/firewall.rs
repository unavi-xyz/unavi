use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
};

use bevy::prelude::*;
use hsd::id::DocId;
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
    Restricted(HashSet<DocId>),
}

impl Default for Access {
    fn default() -> Self {
        Self::Restricted(HashSet::default())
    }
}

impl Access {
    #[must_use]
    pub fn permits(&self, id: &DocId) -> bool {
        match self {
            Self::Open => true,
            Self::Restricted(set) => set.contains(id),
        }
    }
}

impl Firewall {
    /// Every channel open.
    ///
    /// Deliberately not a `Default` impl: openness is a policy a caller has to
    /// ask for by name, and `Access`'s own default is the opposite.
    #[must_use]
    pub fn open() -> Self {
        let mut map = HashMap::new();
        map.insert(Channel::EventRead, Access::Open);
        map.insert(Channel::EventWrite, Access::Open);
        map.insert(Channel::KvRead, Access::Open);
        map.insert(Channel::KvWrite, Access::Open);
        map.insert(Channel::SceneRead, Access::Open);
        map.insert(Channel::SceneWrite, Access::Open);
        Self(Arc::new(RwLock::new(map)))
    }

    /// A document minted by another: its creator alone may write it, while
    /// reads stay open so the rest of the scene can still observe it.
    #[must_use]
    pub fn for_child_doc(creator_id: DocId) -> Self {
        let firewall = Self::open();
        {
            let mut map = firewall.0.write();
            let owner = Access::Restricted(HashSet::from([creator_id]));
            map.insert(Channel::KvWrite, owner.clone());
            map.insert(Channel::SceneWrite, owner);
        }
        firewall
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn access(firewall: &Firewall, channel: Channel) -> Access {
        firewall.0.read().get(&channel).cloned().expect("channel")
    }

    #[test]
    fn child_doc_writes_are_restricted_to_its_creator() {
        let creator = DocId([1; 32]);
        let stranger = DocId([2; 32]);
        let firewall = Firewall::for_child_doc(creator);

        for channel in [Channel::KvWrite, Channel::SceneWrite] {
            let access = access(&firewall, channel);
            assert!(access.permits(&creator), "{channel:?} allows the creator");
            assert!(
                !access.permits(&stranger),
                "{channel:?} must not allow another document"
            );
        }
    }

    #[test]
    fn child_doc_reads_stay_open() {
        let stranger = DocId([2; 32]);
        let firewall = Firewall::for_child_doc(DocId([1; 32]));

        for channel in [Channel::KvRead, Channel::SceneRead, Channel::EventRead] {
            assert!(access(&firewall, channel).permits(&stranger), "{channel:?}");
        }
    }
}
