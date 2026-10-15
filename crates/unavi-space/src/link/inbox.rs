//! Latest value per key, handed from a network task to the ECS.

use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
};

use parking_lot::Mutex;

/// Keys an inbox holds between drains unless told otherwise.
const DEFAULT_CAPACITY: usize = 4096;

/// Latest value per key, handed from a detached network task to the ECS.
///
/// A submission replaces whatever the key held, so a stalled frame costs the
/// intermediate values rather than memory. New keys past the capacity are
/// dropped until the next drain.
pub struct Inbox<K, V>(Arc<Inner<K, V>>);

struct Inner<K, V> {
    map:      Mutex<HashMap<K, V>>,
    capacity: usize,
}

// Manual, not derive: K and V are not Clone.
impl<K, V> Clone for Inbox<K, V> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<K: Eq + Hash, V> Default for Inbox<K, V> {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

impl<K: Eq + Hash, V> Inbox<K, V> {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Arc::new(Inner {
            map: Mutex::new(HashMap::new()),
            capacity,
        }))
    }

    /// Stores `value` under `key`. Returns `false` if the inbox is full and
    /// `key` is new.
    pub fn submit(&self, key: K, value: V) -> bool {
        let mut map = self.0.map.lock();
        if map.len() >= self.0.capacity && !map.contains_key(&key) {
            return false;
        }
        map.insert(key, value);
        true
    }

    /// Takes everything queued, leaving the inbox empty.
    #[must_use]
    pub fn drain(&self) -> HashMap<K, V> {
        std::mem::take(&mut *self.0.map.lock())
    }
}
