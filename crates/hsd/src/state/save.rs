//! Saving is a per-key diff.
//!
//! Because state and the entry set have the same shape, a save writes only the
//! keys that changed. No snapshot, no checkpoint cadence, and no "who
//! checkpoints and when" — two peers editing different prims no longer
//! overwrite each other, which was the defect of a single `snapshot` key.

use std::collections::BTreeMap;

use crate::{
    id::{
        BlobId,
        PrimId,
    },
    key,
    state::entry::EntryValue,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Set {
        key:   String,
        value: Vec<u8>,
    },
    SetBlob {
        key:  String,
        hash: BlobId,
        size: u64,
    },
    /// An empty value at the key. `del` only sweeps the caller's own entries,
    /// so removing a key another peer authored has to be expressed as data.
    Remove {
        key: String,
    },
}

/// The writes that turn `base` into `current`, key-ordered.
#[must_use]
pub fn diff(
    base: &BTreeMap<String, EntryValue>,
    current: &BTreeMap<String, EntryValue>,
) -> Vec<Change> {
    let mut changes = Vec::new();

    for (key, value) in current {
        if base.get(key) == Some(value) {
            continue;
        }
        changes.push(match value {
            EntryValue::Bytes(bytes) => Change::Set {
                key:   key.clone(),
                value: bytes.clone(),
            },
            EntryValue::Blob(bulk) => Change::SetBlob {
                key:  key.clone(),
                hash: bulk.hash,
                size: bulk.size,
            },
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
/// The two prefix wipes sweep every entry this author wrote for the prim; the
/// tombstone is what other peers read. Order matters — `p/<prim>/` prefixes
/// `p/<prim>/parent/`, so wiping afterwards would eat the tombstone.
#[must_use]
pub fn delete_prim(prim: PrimId) -> [Change; 3] {
    [
        Change::Remove {
            key: key::prim_prefix(prim),
        },
        Change::Remove {
            key: key::bulk_prefix(prim),
        },
        Change::Remove {
            key: key::parent(prim),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::entry::BulkRef;

    fn bytes(value: &[u8]) -> EntryValue {
        EntryValue::Bytes(value.to_vec())
    }

    fn base() -> BTreeMap<String, EntryValue> {
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
    fn a_bulk_entry_writes_its_hash() {
        let mut current = base();
        current.insert(
            "b/a/script/".to_owned(),
            EntryValue::Blob(BulkRef {
                hash: BlobId([3; 32]),
                size: 12,
            }),
        );
        assert_eq!(
            diff(&base(), &current),
            vec![Change::SetBlob {
                key:  "b/a/script/".to_owned(),
                hash: BlobId([3; 32]),
                size: 12,
            }]
        );
    }

    #[test]
    fn deleting_a_prim_tombstones_the_parent_last() {
        let prim = PrimId([1; 16]);
        let changes = delete_prim(prim);
        assert_eq!(
            changes[2],
            Change::Remove {
                key: key::parent(prim),
            }
        );
        assert!(key::parent(prim).starts_with(&key::prim_prefix(prim)));
    }
}
