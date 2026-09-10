use std::path::Path;

use anyhow::Context;

/// Namespaces every item against whatever else shares the origin.
const PREFIX: &str = "unavi.";

/// A blocked or missing storage is an error, not an absence.
pub fn read(dir: &Path, key: &str) -> anyhow::Result<Option<String>> {
    storage()?
        .get_item(&item(dir, key))
        .map_err(|_| anyhow::anyhow!("could not read {key} from local storage"))
}

pub fn write(dir: &Path, key: &str, value: &str) -> anyhow::Result<()> {
    storage()?
        .set_item(&item(dir, key), value)
        .map_err(|_| anyhow::anyhow!("could not write {key} to local storage"))
}

/// Records `value` at `key` only if no value sits there yet.
///
/// localStorage has no test-and-set, so this is a check followed by a write,
/// and two tabs racing the same key can both pass. The write itself is atomic,
/// so a loser never tears a winner's value.
pub fn create(dir: &Path, key: &str, value: &str) -> anyhow::Result<()> {
    let storage = storage()?;
    if storage
        .get_item(&item(dir, key))
        .map_err(|_| anyhow::anyhow!("could not read {key} from local storage"))?
        .is_some()
    {
        anyhow::bail!("{key} already exists");
    }
    write(dir, key, value)
}

pub fn read_bytes(dir: &Path, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
    let Some(text) = read(dir, key)? else {
        return Ok(None);
    };
    Ok(Some(
        hex::decode(&text).with_context(|| format!("{key} is not valid hex"))?,
    ))
}

pub fn write_bytes(dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    write(dir, key, &hex::encode(value))
}

/// The browser's local storage, or an error when none is available.
pub fn storage() -> anyhow::Result<web_sys::Storage> {
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
