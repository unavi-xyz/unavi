//! Saving is a per-key diff.
//!
//! Because state and the entry set have the same shape, a save writes only the
//! keys that changed. No snapshot, no checkpoint cadence, and no "who
//! checkpoints and when" — two peers editing different prims no longer
//! overwrite each other, which was the defect of a single `snapshot` key.

use std::collections::BTreeMap;

use crate::{
    id::PrimId,
    key,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Set {
        key:   String,
        value: Vec<u8>,
    },
    /// An empty value at the key. `del` only sweeps the caller's own entries,
    /// so removing a key another peer authored has to be expressed as data.
    Remove {
        key: String,
    },
}

/// The writes that turn `base` into `current`, key-ordered.
#[must_use]
pub fn diff(base: &BTreeMap<String, Vec<u8>>, current: &BTreeMap<String, Vec<u8>>) -> Vec<Change> {
    let mut changes = Vec::new();

    for (key, value) in current {
        if base.get(key) == Some(value) {
            continue;
        }
        changes.push(Change::Set {
            key:   key.clone(),
            value: value.clone(),
        });
    }

    changes.extend(
        base.keys()
            .filter(|key| !current.contains_key(*key))
            .map(|key| Change::Remove { key: key.clone() }),
    );

    changes
}

/// The writes that delete a prim, in the order the format requires.
///
/// The prefix wipe sweeps every entry this author wrote for the prim; the
/// tombstone is what other peers read. Order matters — `p/<prim>/` prefixes
/// `p/<prim>/parent/`, so wiping afterwards would eat the tombstone.
#[must_use]
pub fn delete_prim(prim: PrimId) -> [Change; 2] {
    [
        Change::Remove {
            key: key::prim_prefix(prim),
        },
        Change::Remove {
            key: key::parent(prim),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        id::PrimId,
        key,
    };

    fn bytes(value: &[u8]) -> Vec<u8> {
        value.to_vec()
    }

    fn base() -> BTreeMap<String, Vec<u8>> {
        BTreeMap::from([
            ("p/a/xform/".to_owned(), bytes(&[1])),
            ("p/a/name/".to_owned(), bytes(&[2])),
        ])
    }

    #[test]
    fn an_unchanged_key_is_not_written() {
        assert!(diff(&base(), &base()).is_empty());
    }

    #[test]
    fn a_changed_payload_is_written() {
        let mut current = base();
        current.insert("p/a/xform/".to_owned(), bytes(&[9]));
        assert_eq!(
            diff(&base(), &current),
            vec![Change::Set {
                key:   "p/a/xform/".to_owned(),
                value: vec![9],
            }]
        );
    }

    #[test]
    fn a_dropped_key_is_tombstoned() {
        let mut current = base();
        current.remove("p/a/name/");
        assert_eq!(
            diff(&base(), &current),
            vec![Change::Remove {
                key: "p/a/name/".to_owned(),
            }]
        );
    }

    #[test]
    fn a_slot_entry_writes_its_bytes() {
        let mut current = base();
        current.insert("p/a/script/".to_owned(), vec![9, 9, 9]);
        assert_eq!(
            diff(&base(), &current),
            vec![Change::Set {
                key:   "p/a/script/".to_owned(),
                value: vec![9, 9, 9],
            }]
        );
    }

    #[test]
    fn deleting_a_prim_tombstones_the_parent_last() {
        let prim = PrimId([1; 16]);
        let changes = delete_prim(prim);
        assert_eq!(
            changes[1],
            Change::Remove {
                key: key::parent(prim),
            }
        );
        assert!(key::parent(prim).starts_with(&key::prim_prefix(prim)));
    }
}
