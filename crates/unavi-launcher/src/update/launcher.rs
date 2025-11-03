use std::{
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
    process::Command,
};

use anyhow::{Context, bail};
use self_update::ArchiveKind;
use semver::Version;
use tracing::info;

const USE_BETA: bool = true;

enum SimpleTarget {
    Apple,
    Linux,
    Windows,
}

impl SimpleTarget {
    fn release_str(&self) -> &'static str {
        match self {
            Self::Apple => "macos",
            Self::Linux => "linux",
            Self::Windows => "windows",
        }
    }
}

pub fn update_launcher() -> anyhow::Result<()> {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    let target = self_update::get_target();
    let simple_target = if target.contains("linux") {
        SimpleTarget::Linux
    } else if target.contains("windows") {
        SimpleTarget::Windows
    } else if target.contains("apple") {
        SimpleTarget::Apple
    } else {
        bail!("unsupported platform: {target}")
    };

    info!("Launcher version: {current_version} on {target}");

    let latest_release = self_update::backends::github::ReleaseList::configure()
        .repo_owner("unavi-xyz")
        .repo_name("unavi")
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
        return Ok(());
    }

    info!("Updating to {latest_version}");
    let asset = latest_release
        .assets
        .into_iter()
        .find(|a| a.name.contains("unavi-launcher") && a.name.contains(simple_target.release_str()))
        .ok_or(anyhow::anyhow!("latest asset not found"))?;
    info!("Latest asset: {asset:#?}");

    let tmp_dir = tempfile::Builder::new().prefix("unavi-update").tempdir()?;
    let tmp_archive_path = tmp_dir.path().join(&asset.name);
    let tmp_archive = std::fs::File::create(&tmp_archive_path).context("create archive file")?;
    info!("Downloading to: {}", tmp_archive_path.to_string_lossy());

    self_update::Download::from_url(&asset.download_url)
        .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?)
        .download_to(&tmp_archive)?;

    match simple_target {
        SimpleTarget::Apple | SimpleTarget::Linux => {
            let tmp_tar_path = tmp_dir.path().join(
                asset
                    .name
                    .strip_suffix(".xz")
                    .ok_or(anyhow::anyhow!("invalid asset name (.xz not found)"))?,
            );
            let mut tmp_tar = std::fs::File::create(&tmp_tar_path).context("create tar file")?;

            let tmp_archive =
                std::fs::File::open(&tmp_archive_path).context("reopen archive for reading")?;
            info!(
                "Decoding {} KB archive",
                tmp_archive.metadata()?.size() / 1024
            );

            let mut dec = xz2::read::XzDecoder::new(tmp_archive);
            let mut buf = [0u8; 1024];

            loop {
                let n = dec.read(&mut buf).context("read archive file")?;
                if n == 0 {
                    break;
                }
                tmp_tar.write_all(&buf[0..n]).context("write tar file")?;
            }

            info!("Uncompressed archive: {}", tmp_tar_path.to_string_lossy());

            replace_launcher(&tmp_tar_path, ArchiveKind::Tar(None))?;
        }
        SimpleTarget::Windows => {
            replace_launcher(&tmp_archive_path, ArchiveKind::Zip)?;
        }
    }

    Ok(())
}

fn replace_launcher(path: &Path, archive_kind: ArchiveKind) -> anyhow::Result<()> {
    let out_path = path
        .parent()
        .ok_or(anyhow::anyhow!("extract path has no parent"))?
        .join("out");

    self_update::Extract::from_source(path)
        .archive(archive_kind)
        .extract_into(&out_path)?;

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
