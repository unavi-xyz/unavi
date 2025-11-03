use std::{path::Path, process::Command};

use anyhow::Context;
use self_update::ArchiveKind;
use semver::Version;
use tracing::info;

use super::{
    UpdateStatus,
    common::{
        REPO_NAME, REPO_OWNER, USE_BETA, decompress_xz, extract_archive, get_platform_target,
    },
};

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

    let latest_release = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .with_target(simple_target.release_str())
        .build()?
        .fetch()?
        .into_iter()
        .find(|r| r.version.contains("beta") == USE_BETA)
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

    on_status(UpdateStatus::Downloading(latest_version.to_string()));

    let tmp_dir = tempfile::Builder::new().prefix("unavi-update").tempdir()?;
    let tmp_archive_path = tmp_dir.path().join(&asset.name);
    let tmp_archive = std::fs::File::create(&tmp_archive_path).context("create archive file")?;
    info!("Downloading to: {}", tmp_archive_path.to_string_lossy());

    self_update::Download::from_url(&asset.download_url)
        .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?)
        .download_to(&tmp_archive)?;

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
