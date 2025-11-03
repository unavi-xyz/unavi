use std::{path::Path, process::Command};

use anyhow::Context;
use self_update::ArchiveKind;
use semver::Version;
use tracing::info;

use super::{
    UpdateStatus,
    common::{
        REPO_NAME, REPO_OWNER, decompress_xz, download_with_progress, extract_archive,
        get_platform_target, use_beta,
    },
};

/// Check if an error is a network-related error
fn is_network_error(err: &anyhow::Error) -> bool {
    let err_str = format!("{err:?}").to_lowercase();
    err_str.contains("dns")
        || err_str.contains("connection")
        || err_str.contains("timeout")
        || err_str.contains("network")
        || err_str.contains("unreachable")
}

pub fn update_launcher_with_callback<F>(on_status: F) -> anyhow::Result<()>
where
    F: Fn(UpdateStatus),
{
    on_status(UpdateStatus::Checking);

    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let simple_target = get_platform_target()?;

    info!(
        "Launcher version: {current_version} on {}",
        simple_target.release_str()
    );

    let releases_result = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .with_target(simple_target.release_str())
        .build()
        .and_then(|list| list.fetch());

    let releases = match releases_result {
        Ok(r) => r,
        Err(e) => {
            let err = anyhow::Error::from(e);
            if is_network_error(&err) {
                info!("Network unavailable, skipping launcher update check");
                on_status(UpdateStatus::Offline);
                return Ok(());
            }
            return Err(err);
        }
    };

    let latest_release = releases
        .into_iter()
        .find(|r| r.version.contains("beta") == use_beta())
        .ok_or(anyhow::anyhow!("no valid release found"))?;

    info!("Latest release: {latest_release:#?}");

    let latest_version = Version::parse(&latest_release.version)?;

    if current_version >= latest_version {
        info!("Up to date");
        on_status(UpdateStatus::UpToDate);
        return Ok(());
    }

    info!("Updating to {latest_version}");
    let asset = latest_release
        .assets
        .into_iter()
        .find(|a| a.name.contains("unavi-launcher") && a.name.contains(simple_target.release_str()))
        .ok_or(anyhow::anyhow!("latest asset not found"))?;
    info!("Latest asset: {asset:#?}");

    on_status(UpdateStatus::Downloading {
        version: latest_version.to_string(),
        progress: None,
    });

    let tmp_dir = tempfile::Builder::new().prefix("unavi-update").tempdir()?;
    let tmp_archive_path = tmp_dir.path().join(&asset.name);
    info!("Downloading to: {}", tmp_archive_path.to_string_lossy());

    // Download with progress tracking
    download_with_progress(&asset.download_url, &tmp_archive_path, |progress| {
        on_status(UpdateStatus::Downloading {
            version: latest_version.to_string(),
            progress: Some(progress),
        });
    })?;

    match simple_target {
        super::common::SimpleTarget::Apple | super::common::SimpleTarget::Linux => {
            let tmp_tar_path = tmp_dir.path().join(
                asset
                    .name
                    .strip_suffix(".xz")
                    .ok_or(anyhow::anyhow!("invalid asset name (.xz not found)"))?,
            );

            decompress_xz(&tmp_archive_path, &tmp_tar_path)?;
            info!("Uncompressed archive: {}", tmp_tar_path.to_string_lossy());

            replace_launcher(&tmp_tar_path, ArchiveKind::Tar(None))?;
        }
        super::common::SimpleTarget::Windows => {
            replace_launcher(&tmp_archive_path, ArchiveKind::Zip)?;
        }
    }

    on_status(UpdateStatus::UpdatedNeedsRestart);
    Ok(())
}

fn replace_launcher(path: &Path, archive_kind: ArchiveKind) -> anyhow::Result<()> {
    let out_path = path
        .parent()
        .ok_or(anyhow::anyhow!("extract path has no parent"))?
        .join("out");

    extract_archive(path, archive_kind, &out_path)?;

    for item in std::fs::read_dir(&out_path)? {
        let item = item?;
        let item_name = item.file_name();
        let item_name = item_name.to_string_lossy();

        if item_name.contains("unavi-launcher") {
            info!(
                "Replacing binary with: {}/{item_name}",
                out_path.to_string_lossy()
            );
            let exe = std::env::current_exe()?;
            self_replace::self_replace(item.path())?;
            Command::new(exe).args(std::env::args().skip(1)).spawn()?;
            std::process::exit(0);
        }
    }

    Ok(())
}
