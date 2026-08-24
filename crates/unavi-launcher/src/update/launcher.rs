use std::env;

use anyhow::Context;
use semver::Version;
use tracing::info;

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

pub async fn update_launcher_with_callback<F>(on_status: F) -> anyhow::Result<()>
where
    F: Fn(UpdateStatus) + Send + Sync,
{
    on_status(UpdateStatus::Checking);

    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    info!(
        "Launcher version: {current_version} on {}",
        platform::RELEASE_TARGET
    );

    let latest = match fetch_latest_release().await {
        Ok(r) => r,
        Err(e) => {
            if is_network_error(&e) {
                info!("Network unavailable, skipping launcher update check");
                on_status(UpdateStatus::Offline);
                return Ok(());
            }
            return Err(e);
        }
    };

    if !needs_update(&current_version, &latest.version) {
        info!("Up to date");
        on_status(UpdateStatus::UpToDate);
        return Ok(());
    }

    info!("Updating to {}", latest.version);
    let asset = find_asset(latest.assets, "unavi-launcher", platform::LAUNCHER_EXT)
        .context("launcher asset not found in release")?;

    on_status(UpdateStatus::Downloading {
        version:  latest.version.to_string(),
        progress: None,
    });

    let tmp_dir = tempfile::Builder::new().prefix("unavi-update").tempdir()?;
    let tmp_archive_path = tmp_dir.path().join(&asset.name);
    info!("Downloading to: {}", tmp_archive_path.display());

    download_with_progress(&asset.browser_download_url, &tmp_archive_path, |progress| {
        on_status(UpdateStatus::Downloading {
            version:  latest.version.to_string(),
            progress: Some(progress),
        });
    })
    .await?;

    on_status(UpdateStatus::UpdatedNeedsRestart);

    let mut relaunch = platform::install_launcher(&tmp_archive_path)?;

    // `exit` below runs no destructors, so the download has to be cleaned up
    // by hand once the installed copy no longer reads from it.
    drop(tmp_dir);

    relaunch
        .args(env::args_os().skip(1))
        .spawn()
        .context("failed to relaunch launcher")?;

    std::process::exit(0);
}
