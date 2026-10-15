//! The in-memory backend.

use std::{
    collections::hash_map::Entry,
    sync::{
        Mutex,
        MutexGuard,
    },
};

use super::Map;

pub fn read_bytes(map: &Mutex<Map>, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
    Ok(lock(map)?.get(key).cloned())
}

pub fn write_bytes(map: &Mutex<Map>, key: &str, value: &[u8]) -> anyhow::Result<()> {
    lock(map)?.insert(key.to_string(), value.to_vec());
    Ok(())
}

pub fn create_bytes(map: &Mutex<Map>, key: &str, value: &[u8]) -> anyhow::Result<()> {
    match lock(map)?.entry(key.to_string()) {
        Entry::Vacant(entry) => {
            entry.insert(value.to_vec());
            Ok(())
        }
        Entry::Occupied(_) => anyhow::bail!("{key} already exists"),
    }
}

fn lock(map: &Mutex<Map>) -> anyhow::Result<MutexGuard<'_, Map>> {
    map.lock()
        .map_err(|_| anyhow::anyhow!("storage lock poisoned"))
}
