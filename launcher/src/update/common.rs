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
use futures::{
    Stream,
    StreamExt,
};
use semver::Version;
use serde::Deserialize;

use super::platform::RELEASE_TARGET;

const REPO_OWNER: &str = "unavi-xyz";
const REPO_NAME: &str = "unavi";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Generous enough for a client bundle, but still bounds a server that tries
/// to stream unbounded data at a downloader.
pub const MAX_DOWNLOAD_BYTES: u64 = 1024 * 1024 * 1024;

const GITHUB_HOST: &str = "github.com";

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!("unavi-launcher/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .expect("reqwest client built from static settings")
});

/// A downgrade is refused by construction: `current < latest` is false when
/// `latest` is older than or equal to what is already installed.
pub fn needs_update(current: &Version, latest: &Version) -> bool {
    current < latest
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name:   String,
    #[serde(default)]
    draft:      bool,
    #[serde(default)]
    prerelease: bool,
    assets:     Vec<GitHubAsset>,
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

fn parse_release_version(tag_name: &str) -> Option<Version> {
    Version::parse(tag_name.strip_prefix('v').unwrap_or(tag_name)).ok()
}

/// `None` on a 404 (no releases at all), an error on any other failure.
async fn get_release(url: &str) -> anyhow::Result<Option<GitHubRelease>> {
    let response = HTTP
        .get(url)
        .send()
        .await
        .context("failed to fetch release")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        bail!("GitHub API returned status: {}", response.status());
    }

    response
        .json()
        .await
        .context("failed to parse release JSON")
        .map(Some)
}

/// Scans every release, keeping only non-draft, non-prerelease ones with a
/// tag that parses as semver (any other tag is skipped, not an abort), and
/// picks the highest version rather than trusting creation-date ordering.
async fn fetch_newest_stable_release() -> anyhow::Result<Release> {
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

    releases
        .into_iter()
        .filter(|release| !release.draft && !release.prerelease)
        .filter_map(|release| {
            parse_release_version(&release.tag_name).map(|version| Release {
                version,
                assets: release.assets,
            })
        })
        .max_by(|a, b| a.version.cmp(&b.version))
        .context("no stable release with a valid version tag found")
}

/// Prefers GitHub's own `/releases/latest`, which already excludes drafts
/// and prereleases server-side. Falls back to scanning the full list only if
/// that endpoint has nothing usable (no releases yet, or a non-semver tag),
/// so a single bad release never blocks an update outright.
pub async fn fetch_latest_release() -> anyhow::Result<Release> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest");

    if let Some(release) = get_release(&url).await?
        && let Some(version) = parse_release_version(&release.tag_name)
    {
        return Ok(Release {
            version,
            assets: release.assets,
        });
    }

    fetch_newest_stable_release().await
}

/// Assets for every platform share a release, so both the binary name and the
/// packaging this platform can actually install have to match.
pub fn find_asset<'a>(
    assets: &'a [GitHubAsset],
    binary: &str,
    ext: &str,
) -> Option<&'a GitHubAsset> {
    assets.iter().find(|asset| {
        asset.name.contains(binary)
            && asset.name.contains(RELEASE_TARGET)
            && Path::new(&asset.name)
                .extension()
                .is_some_and(|found| found.eq_ignore_ascii_case(ext))
    })
}

/// The trust root for every download is "GitHub account + release assets";
/// refusing any other host keeps a redirect or a tampered API response from
/// pointing the downloader somewhere else.
fn ensure_github_url(url: &str) -> anyhow::Result<()> {
    let parsed = reqwest::Url::parse(url).with_context(|| format!("invalid asset URL: {url}"))?;
    if parsed.scheme() != "https" || parsed.host_str() != Some(GITHUB_HOST) {
        bail!("refusing to download from non-GitHub host: {url}");
    }
    Ok(())
}

/// Streams `stream` into `dest_path`, aborting if more than `max_bytes` is
/// written or if the final size disagrees with `total_size` (the response's
/// `Content-Length`, or 0 if absent). Generic over the chunk and error type
/// so the capping logic can be exercised with a local stream in tests,
/// without any network.
async fn write_capped_stream<S, C, E, F>(
    mut stream: S,
    dest_path: &Path,
    total_size: u64,
    max_bytes: u64,
    on_progress: F,
) -> anyhow::Result<()>
where
    S: Stream<Item = Result<C, E>> + Unpin,
    C: AsRef<[u8]>,
    E: std::error::Error + Send + Sync + 'static,
    F: Fn(f32),
{
    let mut downloaded: u64 = 0;
    let mut reported = 0;

    let mut file = BufWriter::new(fs::File::create(dest_path).context("failed to create file")?);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(anyhow::Error::from)
            .context("failed to read chunk from response")?;
        let bytes = chunk.as_ref();

        downloaded += bytes.len() as u64;
        if downloaded > max_bytes {
            bail!("download exceeded the {max_bytes}-byte limit");
        }

        file.write_all(bytes).context("failed to write to file")?;

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

    if total_size != 0 && downloaded != total_size {
        bail!(
            "downloaded {downloaded} bytes, expected {total_size} from Content-Length \
             (truncated or tampered response)"
        );
    }

    Ok(())
}

pub async fn download_with_progress<F>(
    url: &str,
    dest_path: &Path,
    max_bytes: u64,
    on_progress: F,
) -> anyhow::Result<()>
where
    F: Fn(f32),
{
    ensure_github_url(url)?;

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
    write_capped_stream(
        response.bytes_stream(),
        dest_path,
        total_size,
        max_bytes,
        on_progress,
    )
    .await
}

pub fn is_network_error(err: &anyhow::Error) -> bool {
    err.chain()
        .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .any(|cause| cause.is_connect() || cause.is_timeout() || cause.is_request())
}

#[cfg(test)]
mod tests {
    use std::io;

    use futures::stream;

    use super::*;

    #[tokio::test]
    async fn writes_full_download_within_cap() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dest = dir.path().join("out.bin");

        let chunks: Vec<Result<Vec<u8>, io::Error>> = (0..5).map(|_| Ok(vec![1u8; 10])).collect();
        write_capped_stream(stream::iter(chunks), &dest, 50, 1000, |_| {})
            .await
            .expect("download within the cap should succeed");

        assert_eq!(fs::read(&dest).expect("read written file").len(), 50);
    }

    #[tokio::test]
    async fn rejects_download_over_the_cap() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dest = dir.path().join("out.bin");

        let chunks: Vec<Result<Vec<u8>, io::Error>> = (0..20).map(|_| Ok(vec![0u8; 10])).collect();
        let err = write_capped_stream(stream::iter(chunks), &dest, 0, 100, |_| {})
            .await
            .expect_err("download over the cap should be rejected");

        assert!(err.to_string().contains("limit"));
    }

    #[tokio::test]
    async fn rejects_length_mismatch_against_content_length() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dest = dir.path().join("out.bin");

        let chunks = vec![Ok::<Vec<u8>, io::Error>(vec![0u8; 10])];
        let err = write_capped_stream(stream::iter(chunks), &dest, 20, 1000, |_| {})
            .await
            .expect_err("a short download should be rejected");

        assert!(err.to_string().contains("expected 20"));
    }

    #[test]
    fn refuses_non_github_download_host() {
        assert!(ensure_github_url("https://evil.example/unavi-client.AppImage").is_err());
        assert!(
            ensure_github_url("https://github.com/unavi-xyz/unavi/releases/download/v1/x").is_ok()
        );
    }
}
