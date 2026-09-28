use std::{
    env,
    path::Path,
    process::Command,
};

use anyhow::Context;
use tracing::info;

#[path = "archive.rs"] mod archive;

pub const RELEASE_TARGET: &str = "windows";
pub const CLIENT_EXE: &str = "unavi-client.exe";
pub const CLIENT_EXT: &str = "xz";
pub const LAUNCHER_EXT: &str = "msi";

/// `msiexec` reports success but defers the file swap to the next boot.
const ERROR_SUCCESS_REBOOT_REQUIRED: i32 = 3010;

pub fn client_command(exe: &Path) -> Command {
    Command::new(exe)
}

pub fn install_client(downloaded: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    archive::unpack_tar_xz(downloaded, dest_dir)
}

pub fn install_launcher(downloaded: &Path) -> anyhow::Result<Command> {
    info!("Installing MSI update: {}", downloaded.display());

    // runas requests the UAC elevation msiexec needs to write Program Files.
    let status = runas::Command::new("msiexec.exe")
        .arg("/i")
        .arg(downloaded)
        .arg("/qn")
        .arg("/norestart")
        .status()
        .context("failed to execute msiexec")?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        if code == ERROR_SUCCESS_REBOOT_REQUIRED {
            info!("MSI installation succeeded, pending reboot");
        } else {
            anyhow::bail!("msiexec failed with exit code: {code}");
        }
    }

    Ok(Command::new(env::current_exe()?))
}
