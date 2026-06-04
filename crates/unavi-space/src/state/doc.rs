use std::collections::HashMap;

use blake3::Hash;
use loro::{
    Container,
    LoroMap,
    LoroValue,
    ValueOrContainer,
};
use loro_surgeon::{
    Hydrate,
    Reconcile,
    bytes::Bytes,
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

pub const DOC_KV_MAX_BYTES: usize = 8 * 1024 * 1024;
pub const KV_KEY_MAX_BYTES: usize = 256;

pub(super) const DOCS_KEY: &str = "docs";
const KV_KEY: &str = "kv";

#[derive(Default, Debug)]
pub struct DocStates(pub HashMap<Hash, DocState>);

impl Hydrate for DocStates {
    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        let mut pairs = Vec::new();
        map.for_each(|k, voc| pairs.push((k.to_string(), voc)));
        let mut out = HashMap::new();
        for (k, voc) in pairs {
            let parsed = k
                .parse::<Hash>()
                .map_err(|_| HydrateError::unexpected("blake3 hash key", "invalid"))?;
            out.insert(parsed, DocState::hydrate(&voc)?);
        }
        Ok(Self(out))
    }
}

impl Reconcile for DocStates {
    type Key = NoKey;

    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        reconcile_keyed_map(&self.0, r)
    }
}

#[derive(Hydrate, Reconcile, Default, Debug)]
#[loro(default)]
pub struct DocState {
    pub kv: HashMap<String, Bytes>,
}

#[derive(Debug, Clone, Copy)]
pub enum KvError {
    KeyTooLong,
    QuotaExceeded,
    Other,
}

pub(super) fn docs_map_mut(root: &SpaceStateRoot) -> Option<LoroMap> {
    let map = root.doc.get_map(ROOT_KEY);
    match map.get_or_create_container(DOCS_KEY, LoroMap::new()) {
        Ok(m) => Some(m),
        Err(err) => {
            warn!(?err, "failed to access docs map");
            None
        }
    }
}

pub(super) fn docs_map_read(root: &SpaceStateRoot) -> Option<LoroMap> {
    let map = root.doc.get_map(ROOT_KEY);
    match map.get(DOCS_KEY)? {
        ValueOrContainer::Container(Container::Map(m)) => Some(m),
        _ => None,
    }
}

fn doc_entry_mut(root: &SpaceStateRoot, doc: Hash) -> Option<LoroMap> {
    let docs = docs_map_mut(root)?;
    match docs.get_or_create_container(&doc.to_string(), LoroMap::new()) {
        Ok(m) => Some(m),
        Err(err) => {
            warn!(?err, "failed to access doc state entry");
            None
        }
    }
}

fn doc_entry_read(root: &SpaceStateRoot, doc: Hash) -> Option<LoroMap> {
    let docs = docs_map_read(root)?;
    match docs.get(&doc.to_string())? {
        ValueOrContainer::Container(Container::Map(m)) => Some(m),
        _ => None,
    }
}

fn kv_map_mut(root: &SpaceStateRoot, doc: Hash) -> Option<LoroMap> {
    let entry = doc_entry_mut(root, doc)?;
    match entry.get_or_create_container(KV_KEY, LoroMap::new()) {
        Ok(m) => Some(m),
        Err(err) => {
            warn!(?err, "failed to access kv map");
            None
        }
    }
}

fn kv_map_read(root: &SpaceStateRoot, doc: Hash) -> Option<LoroMap> {
    let entry = doc_entry_read(root, doc)?;
    match entry.get(KV_KEY)? {
        ValueOrContainer::Container(Container::Map(m)) => Some(m),
        _ => None,
    }
}

#[must_use]
pub fn add_doc(space: Hash, doc: Hash) -> bool {
    let Some(root) = space_state(space) else {
        return false;
    };
    doc_entry_mut(&root, doc).is_some()
}

#[must_use]
pub fn has_doc(space: Hash, doc: Hash) -> bool {
    let Some(root) = space_state(space) else {
        return false;
    };
    let Some(docs) = docs_map_read(&root) else {
        return false;
    };
    docs.get(&doc.to_string()).is_some()
}

#[must_use]
pub fn doc_kv_get(space: Hash, doc: Hash, key: &str) -> Option<Vec<u8>> {
    let root = space_state(space)?;
    let kv = kv_map_read(&root, doc)?;
    match kv.get(key)? {
        ValueOrContainer::Value(LoroValue::Binary(b)) => Some((*b).clone()),
        _ => None,
    }
}

#[must_use]
pub fn doc_kv_keys(space: Hash, doc: Hash) -> Vec<String> {
    let Some(root) = space_state(space) else {
        return Vec::new();
    };
    let Some(kv) = kv_map_read(&root, doc) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    kv.for_each(|k, _| out.push(k.to_string()));
    out
}

pub fn doc_kv_delete(space: Hash, doc: Hash, key: &str) {
    let Some(root) = space_state(space) else {
        return;
    };
    let Some(kv) = kv_map_read(&root, doc) else {
        return;
    };
    if let Err(err) = kv.delete(key) {
        warn!(?err, "failed to delete kv entry");
        return;
    }
    root.doc.commit();
}

pub fn doc_kv_set(space: Hash, doc: Hash, key: &str, value: &[u8]) -> Result<(), KvError> {
    if key.len() > KV_KEY_MAX_BYTES {
        return Err(KvError::KeyTooLong);
    }
    let Some(root) = space_state(space) else {
        return Err(KvError::Other);
    };
    let Some(kv) = kv_map_mut(&root, doc) else {
        return Err(KvError::Other);
    };

    let mut current = 0usize;
    let mut old_value_len = 0usize;
    kv.for_each(|k, voc| {
        let v_len = match voc {
            ValueOrContainer::Value(LoroValue::Binary(b)) => b.len(),
            _ => 0,
        };
        current += k.len() + v_len;
        if k == key {
            old_value_len = v_len;
        }
    });

    let new_total = current
        .saturating_sub(old_value_len)
        .saturating_add(value.len());
    let new_total = if kv.get(key).is_some() {
        new_total
    } else {
        new_total.saturating_add(key.len())
    };
    if new_total > DOC_KV_MAX_BYTES {
        return Err(KvError::QuotaExceeded);
    }

    if let Err(err) = kv.insert(key, LoroValue::Binary(value.to_vec().into())) {
        warn!(?err, "failed to insert kv entry");
        return Err(KvError::Other);
    }
    root.doc.commit();
    Ok(())
}

#[must_use]
pub fn doc_kv_total_bytes(space: Hash, doc: Hash) -> usize {
    let Some(root) = space_state(space) else {
        return 0;
    };
    let Some(kv) = kv_map_read(&root, doc) else {
        return 0;
    };
    let mut total = 0usize;
    kv.for_each(|k, voc| {
        let v_len = match voc {
            ValueOrContainer::Value(LoroValue::Binary(b)) => b.len(),
            _ => 0,
        };
        total += k.len() + v_len;
    });
    total
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

    fn h(seed: &[u8]) -> Hash {
        blake3::hash(seed)
    }

    #[test]
    fn kv_set_get_delete_keys() {
        let space = h(b"kv_set_get_delete_keys-space");
        let doc = h(b"kv_set_get_delete_keys-doc");
        install_test_space(space);
        assert!(add_doc(space, doc));

        assert_eq!(doc_kv_get(space, doc, "foo"), None);
        doc_kv_set(space, doc, "foo", b"bar").expect("set");
        assert_eq!(doc_kv_get(space, doc, "foo").as_deref(), Some(&b"bar"[..]));

        doc_kv_set(space, doc, "baz", b"qux").expect("set");
        let mut keys = doc_kv_keys(space, doc);
        keys.sort();
        assert_eq!(keys, vec!["baz".to_string(), "foo".to_string()]);

        doc_kv_delete(space, doc, "foo");
        assert_eq!(doc_kv_get(space, doc, "foo"), None);
        assert_eq!(doc_kv_keys(space, doc), vec!["baz".to_string()]);
    }

    #[test]
    fn kv_rejects_long_key() {
        let space = h(b"kv_rejects_long_key-space");
        let doc = h(b"kv_rejects_long_key-doc");
        install_test_space(space);
        assert!(add_doc(space, doc));

        let key = "k".repeat(KV_KEY_MAX_BYTES + 1);
        assert!(matches!(
            doc_kv_set(space, doc, &key, b"v"),
            Err(KvError::KeyTooLong)
        ));
    }

    #[test]
    fn kv_rejects_when_over_quota() {
        let space = h(b"kv_rejects_when_over_quota-space");
        let doc = h(b"kv_rejects_when_over_quota-doc");
        install_test_space(space);
        assert!(add_doc(space, doc));

        let big = vec![0u8; DOC_KV_MAX_BYTES - 32];
        doc_kv_set(space, doc, "a", &big).expect("set within cap");
        let result = doc_kv_set(space, doc, "b", &[0u8; 64]);
        assert!(matches!(result, Err(KvError::QuotaExceeded)));
    }

    #[test]
    fn kv_overwrite_does_not_double_count_key() {
        let space = h(b"kv_overwrite-space");
        let doc = h(b"kv_overwrite-doc");
        install_test_space(space);
        assert!(add_doc(space, doc));

        doc_kv_set(space, doc, "k", b"v1").expect("set");
        doc_kv_set(space, doc, "k", b"v2").expect("overwrite");
        assert_eq!(doc_kv_get(space, doc, "k").as_deref(), Some(&b"v2"[..]));
        assert_eq!(doc_kv_total_bytes(space, doc), 1 + 2);
    }
}
