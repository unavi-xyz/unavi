#[cfg(unix)] use std::os::unix::fs::PermissionsExt;
use std::{
    io::{
        self,
        Write,
    },
    path::Path,
};

use anyhow::Context;
use tempfile::{
    Builder,
    NamedTempFile,
};

pub fn read(dir: &Path, key: &str) -> anyhow::Result<Option<String>> {
    match read_bytes(dir, key)? {
        Some(bytes) => Ok(Some(
            String::from_utf8(bytes).context("value is not UTF-8")?,
        )),
        None => Ok(None),
    }
}

pub fn read_bytes(dir: &Path, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
    match std::fs::read(dir.join(key)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("read {key}")),
    }
}

/// Replaces the value at `key` by renaming a fully-written temporary over it,
/// so a crash mid-write leaves whatever was there before rather than a
/// truncated value.
pub fn write_bytes(dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    let path = dir.join(key);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut temp = temp_in(&path).with_context(|| format!("create the temporary for {key}"))?;
    temp.as_file_mut()
        .write_all(value)
        .and_then(|()| temp.as_file().sync_all())
        .with_context(|| format!("write {key}"))?;

    temp.persist(&path)
        .map_err(|err| err.error)
        .with_context(|| format!("replace {key}"))?;
    Ok(())
}

/// Writes `value` at `key` only if no file is there yet, returning an error
/// when one is. The value lands atomically, so a crash never leaves a partial
/// file at `key` — only at worst an inert temporary.
pub fn create(dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    let path = dir.join(key);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut temp = temp_in(&path).with_context(|| format!("create the temporary for {key}"))?;
    temp.as_file_mut()
        .write_all(value)
        .and_then(|()| temp.as_file().sync_all())
        .with_context(|| format!("write {key}"))?;

    temp.persist_noclobber(&path)
        .map_err(|err| err.error)
        .with_context(|| format!("create {key}"))?;
    Ok(())
}

/// A temporary beside `path`, renamed over it only once fully written and
/// synced. Same-directory, so the rename never crosses a filesystem; deleted
/// on drop, so an error path needs no cleanup.
fn temp_in(path: &Path) -> anyhow::Result<NamedTempFile> {
    let parent = path
        .parent()
        .context("a key names a file, not a directory")?;

    let mut builder = Builder::new();

    #[cfg(unix)]
    builder.permissions(std::fs::Permissions::from_mode(0o600));

    Ok(builder.tempfile_in(parent)?)
}
