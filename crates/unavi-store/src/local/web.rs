/// Namespaces every item against whatever else shares the origin.
const PREFIX: &str = "unavi.";

pub fn read(key: &str) -> Option<String> {
    storage().ok()?.get_item(&item(key)).ok()?
}

pub fn write(key: &str, value: &str) -> anyhow::Result<()> {
    storage()?
        .set_item(&item(key), value)
        .map_err(|_| anyhow::anyhow!("could not write {key} to local storage"))
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
