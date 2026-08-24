use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
};

use anyhow::Context;

pub fn set_executable(path: &Path) -> anyhow::Result<()> {
    let mut perms = fs::metadata(path)
        .context("failed to read permissions")?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).context("failed to set permissions")
}
