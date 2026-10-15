//! [`Cells`]: one writer's opinions on one document, and the root hash its
//! signed state commits to.
//!
//! The root hashes each prim's properties, then the list of prims with their
//! hashes. A prim's hash is kept until one of its properties changes, so a
//! tick rehashes only the prims it touched.

use std::collections::BTreeMap;

use hsd::{
    id::PrimId,
    property::name::PropName,
};

use crate::replication::session::SessionKey;

/// Domain separation for a layer's root.
const ROOT_CONTEXT: &[u8] = b"wired/space/layer-root";

/// One prim's opinions, and their hash once computed.
#[derive(Default, Clone)]
struct Prim {
    props: BTreeMap<PropName, Option<Vec<u8>>>,
    hash:  Option<blake3::Hash>,
}

impl Prim {
    fn hash(&mut self, id: PrimId) -> blake3::Hash {
        *self.hash.get_or_insert_with(|| {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&id.0);
            for (name, value) in &self.props {
                hasher.update(&(name.as_str().len() as u64).to_le_bytes());
                hasher.update(name.as_str().as_bytes());
                match value {
                    Some(value) => {
                        hasher.update(&[1]);
                        hasher.update(&(value.len() as u64).to_le_bytes());
                        hasher.update(value);
                    }
                    None => {
                        hasher.update(&[0]);
                    }
                }
            }
            hasher.finalize()
        })
    }
}

/// What a key held before a change.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Previous {
    Absent,
    /// The value it held; `None` is a blocked key.
    Held(Option<Vec<u8>>),
}

/// One writer's opinions on one document. A `None` value blocks its key.
#[derive(Default, Clone)]
pub struct Cells {
    prims: BTreeMap<PrimId, Prim>,
    bytes: u64,
}

impl Cells {
    /// Bytes the cells cost their writer: every name and value.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.prims.is_empty()
    }

    pub(crate) fn get(&self, key: &SessionKey) -> Option<&Option<Vec<u8>>> {
        self.prims.get(&key.prim)?.props.get(&key.name)
    }

    /// Every cell, in key order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (SessionKey, Option<&[u8]>)> + '_ {
        self.prims.iter().flat_map(|(prim, cells)| {
            cells.props.iter().map(|(name, value)| {
                let key = SessionKey {
                    prim: *prim,
                    name: name.clone(),
                };
                (key, value.as_deref())
            })
        })
    }

    /// Sets `key`, answering what it held.
    pub(crate) fn set(&mut self, key: SessionKey, value: Option<Vec<u8>>) -> Previous {
        let added = cell_bytes(&key, value.as_deref());
        let prim = self.prims.entry(key.prim).or_default();
        prim.hash = None;
        let old = prim.props.insert(key.name.clone(), value);
        self.bytes = self.bytes.saturating_add(added);
        match old {
            Some(old) => {
                self.bytes = self.bytes.saturating_sub(cell_bytes(&key, old.as_deref()));
                Previous::Held(old)
            }
            None => Previous::Absent,
        }
    }

    /// Removes `key`, answering what it held.
    pub(crate) fn remove(&mut self, key: &SessionKey) -> Previous {
        let Some(prim) = self.prims.get_mut(&key.prim) else {
            return Previous::Absent;
        };
        let Some(old) = prim.props.remove(&key.name) else {
            return Previous::Absent;
        };
        prim.hash = None;
        if prim.props.is_empty() {
            self.prims.remove(&key.prim);
        }
        self.bytes = self.bytes.saturating_sub(cell_bytes(key, old.as_deref()));
        Previous::Held(old)
    }

    /// The hash a signed state commits these cells to.
    pub fn root(&mut self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(ROOT_CONTEXT);
        for (id, prim) in &mut self.prims {
            hasher.update(&id.0);
            hasher.update(prim.hash(*id).as_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}

/// Bytes one opinion costs its writer.
pub(crate) fn cell_bytes(key: &SessionKey, value: Option<&[u8]>) -> u64 {
    (key.name.as_str().len() + value.map_or(0, <[u8]>::len)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(prim: u8, name: &str) -> SessionKey {
        SessionKey {
            prim: PrimId([prim; 16]),
            name: name.parse().expect("name"),
        }
    }

    #[test]
    fn the_root_depends_on_every_cell_and_not_on_write_order() {
        let mut forward = Cells::default();
        forward.set(key(1, "state/a"), Some(vec![1]));
        forward.set(key(2, "state/b"), None);
        let mut backward = Cells::default();
        backward.set(key(2, "state/b"), None);
        backward.set(key(1, "state/a"), Some(vec![1]));
        assert_eq!(forward.root(), backward.root());

        let before = forward.root();
        forward.set(key(1, "state/a"), Some(vec![2]));
        assert_ne!(forward.root(), before, "a changed value moves the root");
        forward.set(key(1, "state/a"), Some(vec![1]));
        assert_eq!(forward.root(), before, "a cached prim hash is not stale");

        let mut blocked = backward.clone();
        blocked.set(key(1, "state/a"), None);
        assert_ne!(
            blocked.root(),
            before,
            "a blocked key differs from a set one"
        );
        assert_eq!(blocked.remove(&key(1, "state/a")), Previous::Held(None));
        assert_ne!(blocked.root(), before, "a removed key differs from either");
    }

    #[test]
    fn bytes_follow_every_set_and_remove() {
        let mut cells = Cells::default();
        cells.set(key(1, "state/a"), Some(vec![0; 10]));
        assert_eq!(cells.bytes(), 17);
        cells.set(key(1, "state/a"), Some(vec![0; 3]));
        assert_eq!(cells.bytes(), 10);
        cells.remove(&key(1, "state/a"));
        assert_eq!(cells.bytes(), 0);
        assert!(cells.is_empty());
    }
}
