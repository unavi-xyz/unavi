use std::{
    collections::hash_map::Entry,
    sync::{
        Mutex,
        MutexGuard,
    },
};

use anyhow::Context;

use super::Map;

pub fn read(map: &Mutex<Map>, key: &str) -> anyhow::Result<Option<String>> {
    match read_bytes(map, key)? {
        Some(bytes) => Ok(Some(
            String::from_utf8(bytes).context("value is not UTF-8")?,
        )),
        None => Ok(None),
    }
}

pub fn read_bytes(map: &Mutex<Map>, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
    Ok(lock(map)?.get(key).cloned())
}

pub fn write_bytes(map: &Mutex<Map>, key: &str, value: &[u8]) -> anyhow::Result<()> {
    lock(map)?.insert(key.to_string(), value.to_vec());
    Ok(())
}

pub fn create(map: &Mutex<Map>, key: &str, value: &[u8]) -> anyhow::Result<()> {
    let mut map = lock(map)?;
    match map.entry(key.to_string()) {
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
