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
        MAX_DOWNLOAD_BYTES,
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

fn apps_dir() -> PathBuf {
    DIRS.data_local_dir().join("apps")
}

fn current_version_file() -> PathBuf {
    DIRS.data_local_dir().join("current_app_version.txt")
}

fn get_installed_version() -> Option<Version> {
    std::fs::read_to_string(current_version_file())
        .ok()
        .and_then(|s| Version::parse(s.trim()).ok())
}

pub fn installed_app_version() -> Option<String> {
    get_installed_version().map(|v| v.to_string())
}

fn set_installed_version(version: &Version) -> anyhow::Result<()> {
    std::fs::write(current_version_file(), version.to_string())?;
    Ok(())
}

fn app_dir(version: &Version) -> PathBuf {
    apps_dir().join(version.to_string())
}

fn app_exe_path(version: &Version) -> PathBuf {
    app_dir(version).join(platform::APP_EXE)
}

pub fn launch_app() -> anyhow::Result<()> {
    let version = get_installed_version().ok_or_else(|| anyhow::anyhow!("no app installed"))?;
    let exe_path = app_exe_path(&version);
    if !exe_path.exists() {
        anyhow::bail!("app executable not found at {}", exe_path.display());
    }

    info!(
        "Launching app version {version} from {}",
        exe_path.display()
    );
    let mut cmd = platform::app_command(&exe_path, &version)?;
    if crate::CONFIG.get().xr_mode {
        cmd.arg("--xr");
    }
    let child = cmd.spawn().context("failed to launch app")?;
    crate::APP_PROCESS.set(child);

    Ok(())
}

pub async fn update_app_with_callback<F>(on_status: F) -> anyhow::Result<()>
where
    F: Fn(UpdateStatus) + Send + Sync,
{
    on_status(UpdateStatus::Checking);

    let latest = match fetch_latest_release().await {
        Ok(r) => r,
        Err(e) => {
            // Offline is survivable only with an app already installed;
            // otherwise there is nothing to launch.
            if is_network_error(&e) && get_installed_version().is_some() {
                info!("Network unavailable, skipping update check");
                on_status(UpdateStatus::Offline);
                return Ok(());
            }
            return Err(e);
        }
    };

    info!("Latest app release: {}", latest.version);

    let installed_version = get_installed_version();
    info!("Installed app version: {installed_version:?}");

    if let Some(current) = &installed_version
        && !needs_update(current, &latest.version)
    {
        info!("App is up to date");
        on_status(UpdateStatus::UpToDate);
        return Ok(());
    }

    info!("Updating app to {}", latest.version);

    let asset = find_asset(&latest.assets, "unavi-app", platform::APP_EXT)
        .context("app asset not found in release")?;

    on_status(UpdateStatus::Downloading {
        version:  latest.version.to_string(),
        progress: None,
    });

    let tmp_dir = tempfile::Builder::new()
        .prefix("unavi-app-update")
        .tempdir()?;
    let tmp_archive_path = tmp_dir.path().join(&asset.name);
    info!("Downloading app to: {}", tmp_archive_path.display());

    download_with_progress(
        &asset.browser_download_url,
        &tmp_archive_path,
        MAX_DOWNLOAD_BYTES,
        |progress| {
            on_status(UpdateStatus::Downloading {
                version:  latest.version.to_string(),
                progress: Some(progress),
            });
        },
    )
    .await?;

    let dest_dir = app_dir(&latest.version);
    std::fs::create_dir_all(&dest_dir)?;
    platform::install_app(&tmp_archive_path, &dest_dir)?;

    set_installed_version(&latest.version)?;
    info!("App updated to {}", latest.version);

    clean_old_versions(&latest.version, KEEP_VERSIONS)?;

    on_status(UpdateStatus::UpToDate);
    Ok(())
}

fn clean_old_versions(current: &Version, keep_count: usize) -> anyhow::Result<()> {
    let mut versions = std::fs::read_dir(apps_dir())?
        .filter_map(Result::ok)
        .filter_map(|entry| Version::parse(&entry.file_name().to_string_lossy()).ok())
        .collect::<Vec<_>>();

    versions.sort_by(|a, b| b.cmp(a));

    for version in versions.iter().skip(keep_count) {
        if version != current {
            let dir_to_remove = app_dir(version);
            info!("Removing old app version: {version}");
            if let Err(e) = std::fs::remove_dir_all(&dir_to_remove) {
                warn!("Failed to remove old version {version}: {e}");
            }
        }
    }

    Ok(())
}
