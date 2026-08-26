use std::path::Path;

#[must_use]
pub fn read(dir: &Path, key: &str) -> Option<String> {
    std::fs::read_to_string(dir.join(key)).ok()
}

pub fn write(dir: &Path, key: &str, value: &str) -> anyhow::Result<()> {
    let path = dir.join(key);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, value)?;
    Ok(())
}
