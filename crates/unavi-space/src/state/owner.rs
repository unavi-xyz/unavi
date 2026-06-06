use std::collections::HashMap;

use blake3::Hash;
use loro::{
    Container,
    LoroMap,
    ValueOrContainer,
};
use loro_surgeon::{
    Hydrate,
    Reconcile,
    bytes::ByteArray,
    error::{
        HydrateError,
        ReconcileError,
    },
    reconcile::{
        NoKey,
        Reconciler,
        map::reconcile_keyed_map,
    },
};
use tracing::warn;

use crate::state::space::{
    ROOT_KEY,
    SpaceStateRoot,
    space_state,
};

pub(super) const OWNERS_KEY: &str = "owners";

#[derive(Default, Debug)]
pub struct DocOwners(pub HashMap<Hash, OwnerEntry>);

impl Hydrate for DocOwners {
    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        let mut pairs = Vec::new();
        map.for_each(|k, voc| pairs.push((k.to_string(), voc)));
        let mut out = HashMap::new();
        for (k, voc) in pairs {
            let parsed = k
                .parse::<Hash>()
                .map_err(|_| HydrateError::unexpected("blake3 hash key", "invalid"))?;
            out.insert(parsed, OwnerEntry::hydrate(&voc)?);
        }
        Ok(Self(out))
    }
}

impl Reconcile for DocOwners {
    type Key = NoKey;

    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        reconcile_keyed_map(&self.0, r)
    }
}

#[derive(Hydrate, Reconcile, Debug, Clone)]
#[loro(default)]
pub struct OwnerEntry {
    pub peer: ByteArray<32>,
}

impl Default for OwnerEntry {
    fn default() -> Self {
        Self {
            peer: ByteArray::new([0u8; 32]),
        }
    }
}

fn owners_map_mut(root: &SpaceStateRoot) -> Option<LoroMap> {
    let map = root.doc.get_map(ROOT_KEY);
    match map.get_or_create_container(OWNERS_KEY, LoroMap::new()) {
        Ok(m) => Some(m),
        Err(err) => {
            warn!(?err, "failed to access owners map");
            None
        }
    }
}

fn owners_map_read(root: &SpaceStateRoot) -> Option<LoroMap> {
    let map = root.doc.get_map(ROOT_KEY);
    match map.get(OWNERS_KEY)? {
        ValueOrContainer::Container(Container::Map(m)) => Some(m),
        _ => None,
    }
}

fn entry(root: &SpaceStateRoot, doc: Hash) -> Option<LoroMap> {
    let owners = owners_map_mut(root)?;
    match owners.get_or_create_container(&doc.to_string(), LoroMap::new()) {
        Ok(m) => Some(m),
        Err(err) => {
            warn!(?err, "failed to access owner entry");
            None
        }
    }
}

/// Records `peer` as the owner of `doc` in the local replica.
pub fn set_doc_owner(space: Hash, doc: Hash, peer: [u8; 32]) {
    if let Some(existing) = doc_owner(space, doc)
        && existing != [0u8; 32]
        && existing != peer
    {
        warn!(
            doc = %doc,
            "refusing to locally overwrite owner held by another peer",
        );
        return;
    }
    let Some(root) = space_state(space) else {
        return;
    };
    let Some(entry) = entry(&root, doc) else {
        return;
    };
    let attr = OwnerEntry {
        peer: ByteArray::new(peer),
    };
    if let Err(err) = attr.reconcile(loro_surgeon::reconcile::RootReconciler::new(entry)) {
        warn!(?err, "failed to reconcile owner entry");
        return;
    }
    root.doc.commit();
    crate::membership::DOC_SPACE_REGISTRY
        .write()
        .insert(doc, space);
}

#[must_use]
pub fn doc_owner(space: Hash, doc: Hash) -> Option<[u8; 32]> {
    let root = space_state(space)?;
    let owners = owners_map_read(&root)?;
    let voc = owners.get(&doc.to_string())?;
    let ValueOrContainer::Container(Container::Map(map)) = voc else {
        return None;
    };
    OwnerEntry::hydrate_map(&map).ok().map(|e| e.peer.0)
}

#[must_use]
pub fn is_self_doc_owner(space: Hash, doc: Hash) -> bool {
    let Some(me) = crate::peer::self_peer_id() else {
        return false;
    };
    doc_owner(space, doc).is_some_and(|p| p == me)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use loro::LoroDoc;
    use loro_surgeon::reconcile::RootReconciler;

    use super::*;
    use crate::state::space::{
        SPACE_STATES,
        SpaceState,
        SpaceStateRoot,
    };

    fn install_test_space(space: Hash) {
        let doc = LoroDoc::new();
        let map = doc.get_map(ROOT_KEY);
        SpaceState::default()
            .reconcile(RootReconciler::new(map))
            .expect("reconcile state");
        let sub = doc.subscribe_local_update(Box::new(move |_| true));
        SPACE_STATES
            .lock()
            .insert(space, Arc::new(SpaceStateRoot::new(Arc::new(doc), sub)));
    }

    #[test]
    fn set_and_read_owner() {
        let space = blake3::hash(b"owner-space");
        let doc = blake3::hash(b"owner-doc");
        let peer = [7u8; 32];
        install_test_space(space);
        crate::peer::set_self_peer_id(peer);

        assert_eq!(doc_owner(space, doc), None);
        set_doc_owner(space, doc, peer);
        assert_eq!(doc_owner(space, doc), Some(peer));
        assert!(is_self_doc_owner(space, doc));

        let other = [9u8; 32];
        set_doc_owner(space, doc, other);
        assert_eq!(
            doc_owner(space, doc),
            Some(peer),
            "ownership must not be overwritten by a different peer",
        );
    }

    #[test]
    fn set_owner_refuses_steal() {
        let space = blake3::hash(b"owner-steal-space");
        let doc = blake3::hash(b"owner-steal-doc");
        install_test_space(space);

        let original = [1u8; 32];
        let thief = [2u8; 32];
        set_doc_owner(space, doc, original);
        set_doc_owner(space, doc, thief);
        assert_eq!(doc_owner(space, doc), Some(original));
    }
}
