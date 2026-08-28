use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
};

use parking_lot::Mutex;

/// Latest value per key, handed from a detached network task to the ECS.
///
/// A submission replaces whatever the key held, so a stalled frame costs the
/// intermediate values rather than memory.
pub struct Inbox<K, V>(Arc<Mutex<HashMap<K, V>>>);

// Manual, not derive: K and V are not Clone.
impl<K, V> Clone for Inbox<K, V> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<K: Eq + Hash, V> Default for Inbox<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash, V> Inbox<K, V> {
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn submit(&self, key: K, value: V) {
        self.0.lock().insert(key, value);
    }

    /// Takes everything queued, leaving the inbox empty.
    #[must_use]
    pub fn drain(&self) -> HashMap<K, V> {
        std::mem::take(&mut *self.0.lock())
    }
}
