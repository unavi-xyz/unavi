//! Reconcile struct fields and `HashMap` entries into a [`LoroMap`].

use std::collections::{BTreeMap, HashMap};

use loro::ValueOrContainer;

use crate::{error::ReconcileError, reconcile::{MapReconciler, NoKey, PropReconciler, Reconcile, Reconciler}};

impl MapReconciler {
    pub fn entry<R: Reconcile>(&mut self, key: &str, value: &R) -> Result<(), ReconcileError> {
        let reconciler = PropReconciler::map_put(self.map.clone(), key.to_string());
        value.reconcile(reconciler)
    }

    pub fn delete(&mut self, key: &str) -> Result<(), ReconcileError> {
        self.map.delete(key)?;
        Ok(())
    }

    pub fn retain(&mut self, mut pred: impl FnMut(&str) -> bool) -> Result<(), ReconcileError> {
        let keys_to_delete: Vec<String> = self.keys().filter(|k| !pred(k)).collect();
        for key in keys_to_delete {
            self.map.delete(&key)?;
        }
        Ok(())
    }

    pub fn keys(&self) -> impl Iterator<Item = String> {
        let mut keys = Vec::new();
        self.map.for_each(|key, _| {
            keys.push(key.to_string());
        });
        keys.into_iter()
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<ValueOrContainer> {
        self.map.get(key)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.len() == 0
    }

    pub fn entries(&self) -> impl Iterator<Item = (String, ValueOrContainer)> {
        let mut entries = Vec::new();
        self.map.for_each(|key, voc| {
            entries.push((key.to_string(), voc));
        });
        entries.into_iter()
    }
}

impl<V: Reconcile, S: std::hash::BuildHasher> Reconcile for HashMap<String, V, S> {
    type Key = NoKey;

    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        let mut m = r.map()?;
        for (key, value) in self {
            m.entry(key, value)?;
        }
        let new_keys: std::collections::HashSet<&str> =
            self.keys().map(std::string::String::as_str).collect();
        m.retain(|k| new_keys.contains(k))?;
        Ok(())
    }
}

pub fn reconcile_keyed_map<K, V, R, S>(
    map: &HashMap<K, V, S>,
    r: R,
) -> Result<(), ReconcileError>
where
    K: std::fmt::Display + Eq + std::hash::Hash,
    V: Reconcile,
    R: Reconciler,
    S: std::hash::BuildHasher,
{
    let mut m = r.map()?;
    for (key, value) in map {
        let key_str = key.to_string();
        m.entry(&key_str, value)?;
    }
    let new_keys: std::collections::HashSet<String> =
        map.keys().map(std::string::ToString::to_string).collect();
    m.retain(|k| new_keys.contains(k))?;
    Ok(())
}

impl<V: Reconcile> Reconcile for BTreeMap<String, V> {
    type Key = NoKey;

    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        let mut m = r.map()?;
        for (key, value) in self {
            m.entry(key, value)?;
        }
        let new_keys: std::collections::HashSet<&str> =
            self.keys().map(std::string::String::as_str).collect();
        m.retain(|k| new_keys.contains(k))?;
        Ok(())
    }
}
