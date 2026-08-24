use std::{
    env,
    fs,
    os::unix::fs::PermissionsExt,
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};

use anyhow::Context;
use tracing::{
    info,
    warn,
};

pub const RELEASE_TARGET: &str = "linux";
pub const CLIENT_EXE: &str = "unavi-client.AppImage";
pub const CLIENT_EXT: &str = "AppImage";
pub const LAUNCHER_EXT: &str = "AppImage";

const NIXOS_MARKER: &str = "/etc/NIXOS";
const APPIMAGE_RUN: &str = "appimage-run";

// Set for this process by the AppImage runtime and the Nix wrapper. A nested
// AppImage brings its own closure and sets its own runtime variables, so
// inheriting these points it at libraries that only exist in our mount.
const BUNDLE_ENV: &[&str] = &["APPDIR", "APPIMAGE", "ARGV0", "LD_LIBRARY_PATH", "OWD"];

pub fn client_command(exe: &Path) -> Command {
    launch_command(exe)
}

pub fn install_client(downloaded: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    let exe = dest_dir.join(CLIENT_EXE);
    info!("Installing to: {}", exe.display());
    fs::copy(downloaded, &exe).context("failed to install client")?;
    set_executable(&exe)
}

pub fn install_launcher(downloaded: &Path) -> anyhow::Result<Command> {
    // An AppImage runs its executable from a read-only mount, so the file to
    // swap is the AppImage itself, which the runtime names in `APPIMAGE`.
    let target = match env::var_os("APPIMAGE") {
        Some(path) => PathBuf::from(path),
        None => env::current_exe()?,
    };

    let mut staged = target.clone().into_os_string();
    staged.push(".new");
    let staged = PathBuf::from(staged);

    info!("Replacing launcher: {}", target.display());
    stage(downloaded, &staged).inspect_err(|_| {
        let _ = fs::remove_file(&staged);
    })?;
    fs::rename(&staged, &target).context("failed to replace launcher")?;

    Ok(launch_command(&target))
}

fn stage(downloaded: &Path, staged: &Path) -> anyhow::Result<()> {
    fs::copy(downloaded, staged).context("failed to stage new launcher")?;
    set_executable(staged)
}

fn launch_command(appimage: &Path) -> Command {
    build_command(appimage, appimage_runner(appimage).as_deref())
}

// On NixOS an AppImage chroots into a private /nix without the host's real
// graphics drivers; appimage-run's FHS sandbox exposes them instead.
fn appimage_runner(appimage: &Path) -> Option<PathBuf> {
    if !is_nixos() {
        return None;
    }

    let runner = find_appimage_run();
    if runner.is_none() {
        warn!(
            "{APPIMAGE_RUN} is not on PATH, so {} runs without the host's graphics drivers",
            appimage.display()
        );
    }

    runner
}

fn is_nixos() -> bool {
    Path::new(NIXOS_MARKER).exists()
}

fn find_appimage_run() -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|dir| dir.join(APPIMAGE_RUN))
        .find(|path| path.is_file())
}

fn build_command(appimage: &Path, appimage_run: Option<&Path>) -> Command {
    let mut cmd = appimage_run.map_or_else(
        || Command::new(appimage),
        |runner| {
            let mut cmd = Command::new(runner);
            cmd.arg(appimage);
            cmd
        },
    );

    for key in BUNDLE_ENV {
        cmd.env_remove(key);
    }

    cmd
}

fn set_executable(path: &Path) -> anyhow::Result<()> {
    let mut perms = fs::metadata(path)
        .context("failed to read permissions")?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).context("failed to set permissions")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    const APPIMAGE: &str = "/home/user/.local/share/unavi/clients/1.0.0/unavi-client.AppImage";
    const RUNNER: &str = "/run/current-system/sw/bin/appimage-run";

    #[test]
    fn runs_appimage_directly() {
        let cmd = build_command(Path::new(APPIMAGE), None);
        assert_eq!(cmd.get_program(), OsStr::new(APPIMAGE));
        assert_eq!(cmd.get_args().count(), 0);
    }

    #[test]
    fn routes_through_appimage_run() {
        let cmd = build_command(Path::new(APPIMAGE), Some(Path::new(RUNNER)));
        assert_eq!(cmd.get_program(), OsStr::new(RUNNER));
        assert_eq!(cmd.get_args().collect::<Vec<_>>(), [OsStr::new(APPIMAGE)]);
    }

    #[test]
    fn drops_this_bundles_env() {
        let cmd = build_command(Path::new(APPIMAGE), None);
        let cleared = cmd
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .map(|(key, _)| key)
            .collect::<Vec<_>>();

        for key in BUNDLE_ENV {
            assert!(
                cleared.contains(&OsStr::new(key)),
                "{key} is still inherited"
            );
        }
    }
}
