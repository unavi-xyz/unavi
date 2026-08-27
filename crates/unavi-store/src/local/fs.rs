#[cfg(unix)] use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::OpenOptions,
    io::{
        self,
        Write,
    },
    path::Path,
    sync::atomic::{
        AtomicU64,
        Ordering,
    },
};

use anyhow::Context;

/// The next temporary-file suffix within this process, so parallel writers
/// never collide on one `.tmp` name.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// `Ok(None)` when the file does not exist. Anything else that goes wrong,
/// including a value that is not UTF-8, is an `Err`, so a damaged value is not
/// mistaken for a first run.
pub fn read(dir: &Path, key: &str) -> anyhow::Result<Option<String>> {
    match read_bytes(dir, key)? {
        Some(bytes) => Ok(Some(
            String::from_utf8(bytes).context("value is not UTF-8")?,
        )),
        None => Ok(None),
    }
}

/// `Ok(None)` when the file does not exist. Any other read failure is an
/// `Err`.
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

    let temp = temp_path(&path);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&temp)
        .with_context(|| format!("create the temporary for {key}"))?;

    if let Err(err) = file
        .write_all(value)
        .and_then(|()| file.sync_all())
        .with_context(|| format!("write {key}"))
    {
        drop(file);
        let _ = std::fs::remove_file(&temp);
        return Err(err);
    }
    drop(file);

    std::fs::rename(&temp, &path).with_context(|| format!("replace {key}"))?;
    Ok(())
}

/// Writes `value` at `key` only if no file is there yet, returning an error
/// when one is. `create_new` is the only check-and-write a filesystem offers
/// that does not race.
pub fn create(dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    let path = dir.join(key);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&path)
        .with_context(|| format!("create {key}"))?;

    if let Err(err) = file
        .write_all(value)
        .with_context(|| format!("write {key}"))
    {
        drop(file);
        let _ = std::fs::remove_file(&path);
        return Err(err);
    }
    Ok(())
}

fn temp_path(path: &Path) -> std::path::PathBuf {
    let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".tmp.{}-{seq}", std::process::id()));
    std::path::PathBuf::from(name)
}
