use anyhow::Context;

/// Namespaces every item against whatever else shares the origin.
const PREFIX: &str = "unavi.";

/// `Ok(None)` when no item exists. A blocked or missing storage is an `Err`,
/// never an absence.
pub fn read(key: &str) -> anyhow::Result<Option<String>> {
    storage()?
        .get_item(&item(key))
        .map_err(|_| anyhow::anyhow!("could not read {key} from local storage"))
}

pub fn write(key: &str, value: &str) -> anyhow::Result<()> {
    storage()?
        .set_item(&item(key), value)
        .map_err(|_| anyhow::anyhow!("could not write {key} to local storage"))
}

/// Records `value` at `key` only if no value sits there yet.
///
/// localStorage has no test-and-set, so this is a check followed by a write,
/// and two tabs racing the same key can both pass. The write itself is atomic,
/// so a loser never tears a winner's value.
pub fn create(key: &str, value: &str) -> anyhow::Result<()> {
    let storage = storage()?;
    if storage
        .get_item(&item(key))
        .map_err(|_| anyhow::anyhow!("could not read {key} from local storage"))?
        .is_some()
    {
        anyhow::bail!("{key} already exists");
    }
    write(key, value)
}

/// Values are stored hex-encoded. A string that no longer decodes is an
/// `Err` rather than an absent value, so a caller can tell a lost key from a
/// damaged one.
pub fn read_bytes(key: &str) -> anyhow::Result<Option<Vec<u8>>> {
    let Some(text) = read(key)? else {
        return Ok(None);
    };
    Ok(Some(
        super::decode_hex(&text).with_context(|| format!("{key} is not valid hex"))?,
    ))
}

pub fn write_bytes(key: &str, value: &[u8]) -> anyhow::Result<()> {
    write(key, &super::encode_hex(value))
}

pub fn storage() -> anyhow::Result<web_sys::Storage> {
    web_sys::window()
        .ok_or_else(|| anyhow::anyhow!("no window"))?
        .local_storage()
        .map_err(|_| anyhow::anyhow!("local storage is blocked"))?
        .ok_or_else(|| anyhow::anyhow!("no local storage"))
}

fn item(key: &str) -> String {
    format!("{PREFIX}{key}")
}
