use std::sync::LazyLock;

use blake3::Hash;
use parking_lot::Mutex;
use unavi_quota::Stock;

use crate::{
    peer::self_peer_id,
    quota::document_quota,
    state::peer,
};

pub const KV_KEY_MAX_BYTES: usize = 256;

#[derive(Debug, Clone, Copy)]
pub enum KvError {
    KeyTooLong,
    QuotaExceeded,
    Other,
}

/// Serializes the check-and-insert in `doc_kv_set` so concurrent writers cannot
/// both pass the byte-budget check and overshoot the cap.
static KV_WRITE: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Pins `doc` in the local peer's state. Returns whether a local peer identity
/// exists to author the pin.
#[must_use]
pub fn add_doc(space: Hash, doc: Hash) -> bool {
    if self_peer_id().is_none() {
        return false;
    }
    peer::self_pin(space, doc);
    true
}

#[must_use]
pub fn has_doc(space: Hash, doc: Hash) -> bool {
    peer::has_doc(space, doc)
}

#[must_use]
pub fn doc_kv_get(space: Hash, doc: Hash, key: &str) -> Option<Vec<u8>> {
    peer::kv_get(space, doc, key)
}

#[must_use]
pub fn doc_kv_keys(space: Hash, doc: Hash) -> Vec<String> {
    peer::kv_keys(space, doc)
}

#[must_use]
pub fn doc_kv_total_bytes(space: Hash, doc: Hash) -> usize {
    peer::kv_total_bytes(space, doc)
}

pub fn doc_kv_delete(space: Hash, doc: Hash, key: &str) {
    let _guard = KV_WRITE.lock();
    let (_, old_value_len, present) = peer::self_kv_accounting(doc, key);
    peer::self_kv_delete(space, doc, key);
    if present {
        document_quota(doc).release(Stock::KvMemory, (key.len() + old_value_len) as u64);
    }
}

pub fn doc_kv_set(space: Hash, doc: Hash, key: &str, value: &[u8]) -> Result<(), KvError> {
    if key.len() > KV_KEY_MAX_BYTES {
        return Err(KvError::KeyTooLong);
    }
    if self_peer_id().is_none() {
        return Err(KvError::Other);
    }
    let _guard = KV_WRITE.lock();

    let (current, old_value_len, key_present) = peer::self_kv_accounting(doc, key);
    let new_total = current
        .saturating_sub(old_value_len)
        .saturating_add(value.len());
    let new_total = if key_present {
        new_total
    } else {
        new_total.saturating_add(key.len())
    };

    let quota = document_quota(doc);
    let charge = new_total.saturating_sub(current) as u64;
    let release = current.saturating_sub(new_total) as u64;
    if charge > 0 {
        quota
            .try_charge(Stock::KvMemory, charge)
            .map_err(|_| KvError::QuotaExceeded)?;
    }

    peer::self_kv_set(space, doc, key, value);

    if release > 0 {
        quota.release(Stock::KvMemory, release);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use parking_lot::MutexGuard;
    use unavi_quota::limits::Limits;

    use super::*;
    use crate::peer::set_self_peer_id;

    fn h(seed: &[u8]) -> Hash {
        blake3::hash(seed)
    }

    fn setup(peer: [u8; 32]) -> MutexGuard<'static, ()> {
        let guard = peer::TEST_LOCK.lock();
        peer::reset();
        set_self_peer_id(peer);
        guard
    }

    #[test]
    fn kv_set_get_delete_keys() {
        let _g = setup([1u8; 32]);
        let space = h(b"kv_set_get_delete_keys-space");
        let doc = h(b"kv_set_get_delete_keys-doc");
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
        let _g = setup([2u8; 32]);
        let space = h(b"kv_rejects_long_key-space");
        let doc = h(b"kv_rejects_long_key-doc");
        assert!(add_doc(space, doc));

        let key = "k".repeat(KV_KEY_MAX_BYTES + 1);
        assert!(matches!(
            doc_kv_set(space, doc, &key, b"v"),
            Err(KvError::KeyTooLong)
        ));
    }

    #[test]
    fn kv_rejects_when_over_quota() {
        let _g = setup([3u8; 32]);
        let space = h(b"kv_rejects_when_over_quota-space");
        let doc = h(b"kv_rejects_when_over_quota-doc");
        assert!(add_doc(space, doc));

        let cap = *Limits::document()
            .stock
            .get(&Stock::KvMemory)
            .expect("document caps kv memory") as usize;
        let big = vec![0u8; cap - 32];
        doc_kv_set(space, doc, "a", &big).expect("set within cap");
        let result = doc_kv_set(space, doc, "b", &[0u8; 64]);
        assert!(matches!(result, Err(KvError::QuotaExceeded)));
    }

    #[test]
    fn kv_overwrite_does_not_double_count_key() {
        let _g = setup([4u8; 32]);
        let space = h(b"kv_overwrite-space");
        let doc = h(b"kv_overwrite-doc");
        assert!(add_doc(space, doc));

        doc_kv_set(space, doc, "k", b"v1").expect("set");
        doc_kv_set(space, doc, "k", b"v2").expect("overwrite");
        assert_eq!(doc_kv_get(space, doc, "k").as_deref(), Some(&b"v2"[..]));
        assert_eq!(doc_kv_total_bytes(space, doc), 1 + 2);
    }
}
