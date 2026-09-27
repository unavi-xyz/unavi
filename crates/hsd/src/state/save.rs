use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Set {
        key:   String,
        value: Vec<u8>,
    },
    /// Written as an empty value, since a delete only sweeps the writer's own
    /// entries.
    Remove {
        key: String,
    },
}

/// The per-key writes that turn `base` into `current`, key-ordered.
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(diff(&base(), &base()).len(), 0);
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
    fn a_new_key_writes_its_bytes() {
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
}
