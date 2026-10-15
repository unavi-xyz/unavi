//! The native backend: one file per key beneath a root directory.

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

pub fn read_bytes(dir: &Path, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
    match std::fs::read(dir.join(key)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("read {key}")),
    }
}

pub fn write_bytes(dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    write_atomic(dir, key, value, Mode::Replace)
}

pub fn create_bytes(dir: &Path, key: &str, value: &[u8]) -> anyhow::Result<()> {
    write_atomic(dir, key, value, Mode::CreateNew)
}

#[derive(Clone, Copy)]
enum Mode {
    Replace,
    /// Fails when a value is already there.
    CreateNew,
}

/// Renames a fully written and synced temporary into place, so a crash leaves
/// the old value or the new one, never a truncated one. The parent directory is
/// synced after the rename, so the rename itself survives power loss.
fn write_atomic(dir: &Path, key: &str, value: &[u8], mode: Mode) -> anyhow::Result<()> {
    let path = dir.join(key);
    let parent = path
        .parent()
        .context("a key names a file, not a directory")?;
    create_dirs(parent).with_context(|| format!("create the directory for {key}"))?;

    let mut temp = temp_in(parent).with_context(|| format!("create the temporary for {key}"))?;
    temp.as_file_mut()
        .write_all(value)
        .and_then(|()| temp.as_file().sync_all())
        .with_context(|| format!("write {key}"))?;

    match mode {
        Mode::Replace => temp.persist(&path).map(drop),
        Mode::CreateNew => temp.persist_noclobber(&path).map(drop),
    }
    .map_err(|err| err.error)
    .with_context(|| format!("persist {key}"))?;

    #[cfg(unix)]
    std::fs::File::open(parent)
        .and_then(|parent| parent.sync_all())
        .with_context(|| format!("sync the directory of {key}"))?;

    Ok(())
}

/// Owner-only on unix, since the root holds key material.
fn create_dirs(dir: &Path) -> io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);

    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);

    builder.create(dir)
}

/// A temporary in `dir`, so the rename never crosses a filesystem. Deleted on
/// drop, so an error path needs no cleanup.
fn temp_in(dir: &Path) -> io::Result<NamedTempFile> {
    let mut builder = Builder::new();

    #[cfg(unix)]
    builder.permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600));

    builder.tempfile_in(dir)
}
