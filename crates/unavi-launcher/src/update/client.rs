use std::path::PathBuf;

use anyhow::Context;
use semver::Version;
use tracing::{
    info,
    warn,
};

use super::{
    UpdateStatus,
    common::{
        download_with_progress,
        fetch_latest_release,
        find_asset,
        is_network_error,
        needs_update,
    },
    platform,
};
use crate::DIRS;

const KEEP_VERSIONS: usize = 2;

fn clients_dir() -> PathBuf {
    DIRS.data_local_dir().join("clients")
}

fn current_version_file() -> PathBuf {
    DIRS.data_local_dir().join("current_client_version.txt")
}

fn get_installed_version() -> Option<Version> {
    std::fs::read_to_string(current_version_file())
        .ok()
        .and_then(|s| Version::parse(s.trim()).ok())
}

pub fn installed_client_version() -> Option<String> {
    get_installed_version().map(|v| v.to_string())
}

fn set_installed_version(version: &Version) -> anyhow::Result<()> {
    std::fs::write(current_version_file(), version.to_string())?;
    Ok(())
}

fn client_dir(version: &Version) -> PathBuf {
    clients_dir().join(version.to_string())
}

fn client_exe_path(version: &Version) -> PathBuf {
    client_dir(version).join(platform::CLIENT_EXE)
}

pub fn launch_client() -> anyhow::Result<()> {
    let version = get_installed_version().ok_or_else(|| anyhow::anyhow!("no client installed"))?;
    let exe_path = client_exe_path(&version);
    if !exe_path.exists() {
        anyhow::bail!("client executable not found at {}", exe_path.display());
    }

    info!(
        "Launching client version {version} from {}",
        exe_path.display()
    );
    let mut cmd = platform::client_command(&exe_path);
    if crate::CONFIG.get().xr_mode {
        cmd.arg("--xr");
    }
    let child = cmd.spawn().context("failed to launch client")?;
    crate::CLIENT_PROCESS.set(child);

    Ok(())
}

pub async fn update_client_with_callback<F>(on_status: F) -> anyhow::Result<()>
where
    F: Fn(UpdateStatus) + Send + Sync,
{
    on_status(UpdateStatus::Checking);

    let latest = match fetch_latest_release().await {
        Ok(r) => r,
        Err(e) => {
            // Skipping the check is only safe once a client is already
            // installed; otherwise there is nothing to launch, so a network
            // failure has to surface as an error rather than silently
            // proceeding to a broken home screen.
            if is_network_error(&e) && get_installed_version().is_some() {
                info!("Network unavailable, skipping update check");
                on_status(UpdateStatus::Offline);
                return Ok(());
            }
            return Err(e);
        }
    };

    info!("Latest client release: {}", latest.version);

    let installed_version = get_installed_version();
    info!("Installed client version: {installed_version:?}");

    if let Some(current) = &installed_version
        && !needs_update(current, &latest.version)
    {
        info!("Client is up to date");
        on_status(UpdateStatus::UpToDate);
        return Ok(());
    }

    info!("Updating client to {}", latest.version);
    let asset = find_asset(latest.assets, "unavi-client", platform::CLIENT_EXT)
        .context("client asset not found in release")?;

    on_status(UpdateStatus::Downloading {
        version:  latest.version.to_string(),
        progress: None,
    });

    let tmp_dir = tempfile::Builder::new()
        .prefix("unavi-client-update")
        .tempdir()?;
    let tmp_archive_path = tmp_dir.path().join(&asset.name);
    info!("Downloading client to: {}", tmp_archive_path.display());

    download_with_progress(&asset.browser_download_url, &tmp_archive_path, |progress| {
        on_status(UpdateStatus::Downloading {
            version:  latest.version.to_string(),
            progress: Some(progress),
        });
    })
    .await?;

    let dest_dir = client_dir(&latest.version);
    std::fs::create_dir_all(&dest_dir)?;
    platform::install_client(&tmp_archive_path, &dest_dir)?;

    set_installed_version(&latest.version)?;
    info!("Client updated to {}", latest.version);

    clean_old_versions(&latest.version, KEEP_VERSIONS)?;

    on_status(UpdateStatus::UpToDate);
    Ok(())
}

fn clean_old_versions(current: &Version, keep_count: usize) -> anyhow::Result<()> {
    let mut versions = std::fs::read_dir(clients_dir())?
        .filter_map(Result::ok)
        .filter_map(|entry| Version::parse(&entry.file_name().to_string_lossy()).ok())
        .collect::<Vec<_>>();

    versions.sort_by(|a, b| b.cmp(a));

    for version in versions.iter().skip(keep_count) {
        if version != current {
            let dir_to_remove = client_dir(version);
            info!("Removing old client version: {version}");
            if let Err(e) = std::fs::remove_dir_all(&dir_to_remove) {
                warn!("Failed to remove old version {version}: {e}");
            }
        }
    }

    Ok(())
}
