use std::io::{Read, Write};

use anyhow::{Context, bail};
use self_update::ArchiveKind;

pub const USE_BETA: bool = true;
pub const REPO_OWNER: &str = "unavi-xyz";
pub const REPO_NAME: &str = "unavi";

#[derive(Debug, Clone, Copy)]
pub enum SimpleTarget {
    Apple,
    Linux,
    Windows,
}

impl SimpleTarget {
    pub fn release_str(&self) -> &'static str {
        match self {
            Self::Apple => "macos",
            Self::Linux => "linux",
            Self::Windows => "windows",
        }
    }
}

/// Get the platform target for the current system
pub fn get_platform_target() -> anyhow::Result<SimpleTarget> {
    let target = self_update::get_target();

    if target.contains("linux") {
        Ok(SimpleTarget::Linux)
    } else if target.contains("windows") {
        Ok(SimpleTarget::Windows)
    } else if target.contains("apple") {
        Ok(SimpleTarget::Apple)
    } else {
        bail!("unsupported platform: {target}")
    }
}

/// Decompress an XZ archive to a tar file
pub fn decompress_xz(xz_path: &std::path::Path, tar_path: &std::path::Path) -> anyhow::Result<()> {
    let xz_file = std::fs::File::open(xz_path).context("open xz file")?;

    let mut tar_file = std::fs::File::create(tar_path).context("create tar file")?;
    let mut decoder = xz2::read::XzDecoder::new(xz_file);
    let mut buf = [0u8; 1024];

    loop {
        let n = decoder.read(&mut buf).context("read xz archive")?;
        if n == 0 {
            break;
        }
        tar_file.write_all(&buf[0..n]).context("write tar file")?;
    }

    Ok(())
}

/// Extract an archive to a destination directory
pub fn extract_archive(
    archive_path: &std::path::Path,
    archive_kind: ArchiveKind,
    dest: &std::path::Path,
) -> anyhow::Result<()> {
    self_update::Extract::from_source(archive_path)
        .archive(archive_kind)
        .extract_into(dest)?;
    Ok(())
}
