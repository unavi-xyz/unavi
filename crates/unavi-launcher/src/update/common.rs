use std::{
    fs,
    io::{Read, Write},
};

use anyhow::{Context, bail};
use self_update::ArchiveKind;

pub const REPO_OWNER: &str = "unavi-xyz";
pub const REPO_NAME: &str = "unavi";

pub fn use_beta() -> bool {
    crate::CONFIG.get().update_channel.is_beta()
}

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

/// Download a file with progress reporting
pub fn download_with_progress<F>(
    url: &str,
    dest_path: &std::path::Path,
    on_progress: F,
) -> anyhow::Result<()>
where
    F: Fn(f32),
{
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::default())
        .build()
        .context("failed to build http client")?;
    let mut response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/octet-stream")
        .header(reqwest::header::USER_AGENT, "unavi-launcher")
        .send()
        .context("failed to start download")?;

    // Check if the request was successful
    if !response.status().is_success() {
        anyhow::bail!(
            "download failed with status {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        );
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file = fs::File::create(dest_path).context("failed to create file")?;

    let mut buffer = [0; 8192];
    loop {
        let bytes_read = response
            .read(&mut buffer)
            .context("failed to read from response")?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .context("failed to write to file")?;

        downloaded += bytes_read as u64;

        if total_size > 0 {
            let progress = (downloaded as f32 / total_size as f32) * 100.0;
            on_progress(progress);
        }
    }

    Ok(())
}
