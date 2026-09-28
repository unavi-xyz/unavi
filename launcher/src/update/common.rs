use std::{
    fs,
    io::{
        BufWriter,
        Write,
    },
    path::Path,
    sync::LazyLock,
    time::Duration,
};

use anyhow::{
    Context,
    bail,
};
use futures::StreamExt;
use semver::Version;
use serde::Deserialize;

use super::platform::RELEASE_TARGET;

const REPO_OWNER: &str = "unavi-xyz";
const REPO_NAME: &str = "unavi";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!("unavi-launcher/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .expect("reqwest client built from static settings")
});

pub fn needs_update(current: &Version, latest: &Version) -> bool {
    current < latest
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets:   Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name:                 String,
    pub browser_download_url: String,
}

#[derive(Debug)]
pub struct Release {
    pub version: Version,
    pub assets:  Vec<GitHubAsset>,
}

pub async fn fetch_latest_release() -> anyhow::Result<Release> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases");

    let response = HTTP
        .get(&url)
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

    let release = releases
        .into_iter()
        .find(|r| !r.tag_name.contains("beta"))
        .context("no valid release found")?;

    let version = Version::parse(
        release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name),
    )
    .with_context(|| format!("release tag is not a version: {}", release.tag_name))?;

    Ok(Release {
        version,
        assets: release.assets,
    })
}

/// Assets for every platform share a release, so both the binary name and the
/// packaging this platform can actually install have to match.
pub fn find_asset(assets: Vec<GitHubAsset>, binary: &str, ext: &str) -> Option<GitHubAsset> {
    assets.into_iter().find(|asset| {
        asset.name.contains(binary)
            && asset.name.contains(RELEASE_TARGET)
            && Path::new(&asset.name)
                .extension()
                .is_some_and(|found| found.eq_ignore_ascii_case(ext))
    })
}

pub async fn download_with_progress<F>(
    url: &str,
    dest_path: &Path,
    on_progress: F,
) -> anyhow::Result<()>
where
    F: Fn(f32),
{
    let response = HTTP
        .get(url)
        .header(reqwest::header::ACCEPT, "application/octet-stream")
        .send()
        .await
        .context("failed to start download")?;

    if !response.status().is_success() {
        bail!(
            "download failed with status {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        );
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut reported = 0;

    let mut file = BufWriter::new(fs::File::create(dest_path).context("failed to create file")?);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("failed to read chunk from response")?;
        file.write_all(&chunk).context("failed to write to file")?;

        downloaded += chunk.len() as u64;

        if total_size == 0 {
            continue;
        }

        // At most one progress callback per percent point.
        let percent = (downloaded * 100 / total_size).min(100);
        if percent > reported {
            reported = percent;
            on_progress(percent as f32);
        }
    }

    file.flush().context("failed to flush download")?;

    Ok(())
}

pub fn is_network_error(err: &anyhow::Error) -> bool {
    err.chain()
        .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .any(|cause| cause.is_connect() || cause.is_timeout() || cause.is_request())
}
