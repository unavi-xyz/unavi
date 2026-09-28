use std::{
    env,
    ffi::OsStr,
    fs,
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};

use anyhow::Context;
use tracing::info;

use self::unix::set_executable;

#[path = "archive.rs"] mod archive;
#[path = "unix.rs"] mod unix;

pub const RELEASE_TARGET: &str = "macos";
pub const CLIENT_EXE: &str = "unavi-client";
pub const CLIENT_EXT: &str = "xz";
pub const LAUNCHER_EXT: &str = "xz";

const LAUNCHER_EXE: &str = "unavi-launcher";

pub fn client_command(exe: &Path) -> Command {
    Command::new(exe)
}

pub fn install_client(downloaded: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    archive::unpack_tar_xz(downloaded, dest_dir)?;

    // The bundle inside the archive is named after the build target, not the
    // crate, so the launcher would not find it under the name it looks up.
    let exe = dest_dir.join(CLIENT_EXE);
    if !exe.exists() {
        fs::rename(find_bundled(dest_dir, CLIENT_EXE)?, &exe)
            .context("failed to install client")?;
    }

    set_executable(&exe)
}

pub fn install_launcher(downloaded: &Path) -> anyhow::Result<Command> {
    let out_dir = downloaded
        .parent()
        .context("download path has no parent")?
        .join("out");

    archive::unpack_tar_xz(downloaded, &out_dir)?;

    let replacement = find_bundled(&out_dir, LAUNCHER_EXE)?;
    info!("Replacing launcher with: {}", replacement.display());

    let exe = env::current_exe()?;
    self_replace::self_replace(replacement).context("failed to replace launcher")?;

    Ok(Command::new(exe))
}

fn find_bundled(dir: &Path, prefix: &str) -> anyhow::Result<PathBuf> {
    fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.starts_with(prefix))
        })
        .with_context(|| format!("{prefix} not found in {}", dir.display()))
}
