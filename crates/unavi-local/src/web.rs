//! The wasm backend: browser local storage, which holds only text, so values
//! are stored through [`text`](crate::text).

use std::path::Path;

use crate::text::{
    decode,
    encode,
};

/// Namespaces every item against whatever else shares the origin.
const PREFIX: &str = "unavi.";

pub fn read_bytes(dir: &Path, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
    Ok(get(&storage()?, dir, key)?.map(|text| decode(&text)))
}

pub fn write_bytes(dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    set(&storage()?, dir, key, value)
}

/// localStorage has no test-and-set, so this is a check followed by a write,
/// and two tabs racing the same key can both pass. The write itself is atomic,
/// so a loser never tears a winner's value.
pub fn create_bytes(dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    let storage = storage()?;
    if get(&storage, dir, key)?.is_some() {
        anyhow::bail!("{key} already exists");
    }
    set(&storage, dir, key, value)
}

/// A blocked or missing storage is an error, not an absence.
fn get(storage: &web_sys::Storage, dir: &Path, key: &str) -> anyhow::Result<Option<String>> {
    storage
        .get_item(&item(dir, key))
        .map_err(|_| anyhow::anyhow!("could not read {key} from local storage"))
}

fn set(storage: &web_sys::Storage, dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    storage
        .set_item(&item(dir, key), &encode(value))
        .map_err(|_| anyhow::anyhow!("could not write {key} to local storage"))
}

fn storage() -> anyhow::Result<web_sys::Storage> {
    web_sys::window()
        .ok_or_else(|| anyhow::anyhow!("no window"))?
        .local_storage()
        .map_err(|_| anyhow::anyhow!("local storage is blocked"))?
        .ok_or_else(|| anyhow::anyhow!("no local storage"))
}

/// A `dir` root becomes a prefix inside the one browser-local namespace, so
/// the data and config roots stay apart just as their directories do on
/// native.
fn item(dir: &Path, key: &str) -> String {
    format!("{PREFIX}{}/{}", dir.display(), key)
}
