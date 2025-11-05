use std::{path::PathBuf, process::Command};

use anyhow::Context;
use semver::Version;
use tracing::info;

use super::{
    UpdateStatus,
    common::{
        ArchiveKind, decompress_xz, download_with_progress, extract_archive, fetch_github_releases,
        get_platform_target, is_network_error, use_beta,
    },
};
use crate::DIRS;

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
    DIRS.data_local_dir()
        .join("clients")
        .join(version.to_string())
}

fn client_exe_path(version: &Version) -> PathBuf {
    let exe_name = if cfg!(windows) {
        "unavi-client.exe"
    } else {
        "unavi-client"
    };
    client_dir(version).join(exe_name)
}

pub fn launch_client() -> anyhow::Result<()> {
    // Try versioned client first
    if let Some(version) = get_installed_version() {
        let exe_path = client_exe_path(&version);
        if exe_path.exists() {
            info!(
                "Launching client version {version} from {}",
                exe_path.display()
            );
            let child = Command::new(exe_path)
                .spawn()
                .context("failed to launch client")?;
            crate::CLIENT_PROCESS.set(child);
            return Ok(());
        }
    }

    // Fall back to sibling executable
    let exe_path = std::env::current_exe()?
        .parent()
        .ok_or(anyhow::anyhow!("failed to get executable directory"))?
        .join(if cfg!(windows) {
            "unavi-client.exe"
        } else {
            "unavi-client"
        });

    if !exe_path.exists() {
        anyhow::bail!("client executable not found at {}", exe_path.display());
    }

    info!("Launching client from {}", exe_path.display());
    let child = Command::new(exe_path)
        .spawn()
        .context("failed to launch client")?;
    crate::CLIENT_PROCESS.set(child);

    Ok(())
}

pub async fn update_client_with_callback<F>(on_status: F) -> anyhow::Result<()>
where
    F: Fn(UpdateStatus) + Send + Sync,
{
    on_status(UpdateStatus::Checking);

    let simple_target = get_platform_target()?;

    let releases_result = fetch_github_releases().await;

    let releases = match releases_result {
        Ok(r) => r,
        Err(e) => {
            if is_network_error(&e) {
                info!("Network unavailable, skipping update check");
                on_status(UpdateStatus::Offline);
                return Ok(());
            }
            return Err(e);
        }
    };

    let latest_release = releases
        .into_iter()
        .find(|r| {
            if use_beta() {
                true
            } else {
                !r.tag_name.contains("beta")
            }
        })
        .ok_or(anyhow::anyhow!("no valid release found"))?;

    let latest_version = Version::parse(
        latest_release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&latest_release.tag_name),
    )?;
    info!("Latest client release: {latest_version}");

    let installed_version = get_installed_version();
    info!("Installed client version: {installed_version:?}");

    if let Some(ref current) = installed_version
        && current >= &latest_version
    {
        info!("Client is up to date");
        on_status(UpdateStatus::UpToDate);
        return Ok(());
    }

    info!("Updating client to {latest_version}");
    let asset = latest_release
        .assets
        .into_iter()
        .find(|a| a.name.contains("unavi-client") && a.name.contains(simple_target.release_str()))
        .ok_or(anyhow::anyhow!("client asset not found in release"))?;

    on_status(UpdateStatus::Downloading {
        version: latest_version.to_string(),
        progress: None,
    });

    let tmp_dir = tempfile::Builder::new()
        .prefix("unavi-client-update")
        .tempdir()?;
    let tmp_archive_path = tmp_dir.path().join(&asset.name);
    info!(
        "Downloading client to: {}",
        tmp_archive_path.to_string_lossy()
    );

    // Download with progress tracking
    download_with_progress(&asset.browser_download_url, &tmp_archive_path, |progress| {
        on_status(UpdateStatus::Downloading {
            version: latest_version.to_string(),
            progress: Some(progress),
        });
    })
    .await?;

    let extract_path = client_dir(&latest_version);
    std::fs::create_dir_all(&extract_path)?;

    match simple_target {
        super::common::SimpleTarget::Apple | super::common::SimpleTarget::Linux => {
            let tmp_tar_path = tmp_dir.path().join(
                asset
                    .name
                    .strip_suffix(".xz")
                    .ok_or(anyhow::anyhow!("invalid asset name (.xz not found)"))?,
            );

            decompress_xz(&tmp_archive_path, &tmp_tar_path)?;
            info!("Extracting to: {}", extract_path.display());
            extract_archive(&tmp_tar_path, ArchiveKind::Tar, &extract_path)?;
        }
        super::common::SimpleTarget::Windows => {
            extract_archive(&tmp_archive_path, ArchiveKind::Zip, &extract_path)?;
        }
    }

    // Set executable permissions on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let exe_path = client_exe_path(&latest_version);
        if exe_path.exists() {
            let mut perms = std::fs::metadata(&exe_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&exe_path, perms)?;
        }
    }

    set_installed_version(&latest_version)?;
    info!("Client updated to {latest_version}");

    // Clean up old versions (keep last 2)
    clean_old_versions(&latest_version, 2)?;

    on_status(UpdateStatus::UpToDate);
    Ok(())
}

fn clean_old_versions(current: &Version, keep_count: usize) -> anyhow::Result<()> {
    let clients_dir = DIRS.data_local_dir().join("clients");

    let mut versions: Vec<Version> = std::fs::read_dir(&clients_dir)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            Version::parse(&name_str).ok()
        })
        .collect();

    // Sort versions in descending order (newest first)
    versions.sort_by(|a, b| b.cmp(a));

    // Keep current version plus keep_count-1 older versions
    for version in versions.iter().skip(keep_count) {
        if version != current {
            let dir_to_remove = client_dir(version);
            info!("Removing old client version: {version}");
            if let Err(e) = std::fs::remove_dir_all(&dir_to_remove) {
                tracing::warn!("Failed to remove old version {version}: {e}");
            }
        }
    }

    Ok(())
}
