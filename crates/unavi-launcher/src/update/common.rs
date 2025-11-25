use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, bail};
use futures::StreamExt;
use semver::Version;
use serde::Deserialize;

pub const REPO_OWNER: &str = "unavi-xyz";
pub const REPO_NAME: &str = "unavi";

pub fn is_beta() -> bool {
    crate::CONFIG.get().update_channel.is_beta()
}

/// Check if an update is needed, considering both version and channel.
/// Returns true if update is needed, false if already up to date.
pub fn needs_update(current: &Version, latest: &Version) -> bool {
    let want_beta = is_beta();
    let latest_is_beta = latest.pre.as_str().contains("beta");

    // If we're on the same version, no update needed regardless of channel.
    if current == latest {
        return false;
    }

    // If latest version matches our desired channel, update if it's newer.
    if latest_is_beta == want_beta {
        return current < latest;
    }

    // Latest doesn't match our channel. If we want beta but latest is stable,
    // only update if we're on an older stable version.
    // If we want stable but latest is beta, don't update.
    if want_beta && !latest_is_beta {
        // Want beta, but latest is stable. Update if our version is older.
        return current < latest;
    }

    // Want stable, but latest is beta. Don't update.
    false
}

#[derive(Debug, Clone, Copy)]
pub enum SimpleTarget {
    Apple,
    Linux,
    Windows,
}

impl SimpleTarget {
    pub const fn release_str(self) -> &'static str {
        match self {
            Self::Apple => "macos",
            Self::Linux => "linux",
            Self::Windows => "windows",
        }
    }
}

pub fn get_platform_target() -> anyhow::Result<SimpleTarget> {
    match std::env::consts::OS {
        "linux" => Ok(SimpleTarget::Linux),
        "windows" => Ok(SimpleTarget::Windows),
        "macos" => Ok(SimpleTarget::Apple),
        os => bail!("unsupported platform: {os}"),
    }
}

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub async fn fetch_github_releases() -> anyhow::Result<Vec<GitHubRelease>> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases");

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, "unavi-launcher")
        .send()
        .await
        .context("failed to fetch releases")?;

    if !response.status().is_success() {
        bail!("GitHub API returned status: {}", response.status());
    }

    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .context("failed to parse releases JSON")?;

    Ok(releases)
}

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

pub fn extract_archive(archive_path: &Path, dest: &Path) -> anyhow::Result<()> {
    let tar_file = fs::File::open(archive_path).context("failed to open tar file")?;
    let mut archive = tar::Archive::new(tar_file);
    archive
        .unpack(dest)
        .context("failed to extract tar archive")?;
    Ok(())
}

pub async fn download_with_progress<F>(
    url: &str,
    dest_path: &Path,
    on_progress: F,
) -> anyhow::Result<()>
where
    F: Fn(f32),
{
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/octet-stream")
        .header(reqwest::header::USER_AGENT, "unavi-launcher")
        .send()
        .await
        .context("failed to start download")?;

    // Check if the request was successful
    if !response.status().is_success() {
        bail!(
            "download failed with status {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        );
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file = fs::File::create(dest_path).context("failed to create file")?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("failed to read chunk from response")?;
        file.write_all(&chunk).context("failed to write to file")?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = (downloaded as f32 / total_size as f32) * 100.0;
            on_progress(progress);
        }
    }

    Ok(())
}

pub fn is_network_error(err: &anyhow::Error) -> bool {
    let err_str = format!("{err:?}").to_lowercase();
    err_str.contains("dns")
        || err_str.contains("connection")
        || err_str.contains("timeout")
        || err_str.contains("network")
        || err_str.contains("unreachable")
}
