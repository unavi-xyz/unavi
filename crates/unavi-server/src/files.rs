//! Hosts files from a local directory over iroh blobs.
//!
//! Every file in [`files_dir`] is pinned under a `files/` tag so the store's
//! GC keeps it, and served over `iroh_blobs::ALPN`; a sweep on each host pass
//! drops tags the directory no longer backs. A `files.json` index lists
//! name -> hash so an operator can wire the client's manifest to what this
//! node hosts.

use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use iroh_blobs::api::blobs::Blobs;
use n0_future::StreamExt;
use serde::Serialize;
use tracing::{
    info,
    warn,
};
use wds::DataStore;

use crate::DIRS;

/// The runtime directory holding hosted files, one file per blob.
#[must_use]
pub fn files_dir() -> PathBuf {
    DIRS.data_local_dir().join("files")
}

/// Tag prefix for hosted blobs, so pins are namespaced.
pub const TAG_PREFIX: &str = "files/";

/// Top-level entries describing the hosting itself rather than content to host.
const EXCLUDED: &[&str] = &["README.md", "files.json"];

const README: &str = "Drop files here to host them over iroh-blobs.\n\
                      Everything in this directory is pinned and served, \
                      recursively.\n\
                      A file is hosted under its path relative to this \
                      directory.\n";

/// One hosted file, as recorded in `files.json`.
#[derive(Debug, Clone, Serialize)]
pub struct HostedFile {
    pub name: String,
    pub hash: iroh_blobs::Hash,
    pub size: u64,
}

/// Ensures [`files_dir`] exists, with a README explaining what it is for.
pub fn init_files_dir() -> anyhow::Result<()> {
    init_files(&files_dir())
}

fn init_files(dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("README.md"), README)?;
    Ok(())
}

/// Every hostable file under `dir`, keyed by its path relative to `dir`.
///
/// Symlinks are skipped rather than followed: a link inside the directory must
/// not publish a file outside it.
fn hosted_files(dir: &Path) -> anyhow::Result<Vec<(String, PathBuf)>> {
    let mut found = Vec::new();
    collect_files(dir, dir, &mut found)?;
    found.sort_by(|(a, _), (b, _)| a.cmp(b));
    Ok(found)
}

fn collect_files(
    root: &Path,
    dir: &Path,
    found: &mut Vec<(String, PathBuf)>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_symlink() {
            warn!(path = %path.display(), "not hosting symlink");
            continue;
        }

        if file_type.is_dir() {
            collect_files(root, &path, found)?;
            continue;
        }

        let Some(name) = rel_name(root, &path) else {
            warn!(path = %path.display(), "not hosting file with a non-utf8 name");
            continue;
        };

        if EXCLUDED.contains(&name.as_str()) {
            continue;
        }

        found.push((name, path));
    }
    Ok(())
}

fn rel_name(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let mut name = String::new();
    for part in rel.components() {
        let part = part.as_os_str().to_str()?;
        if !name.is_empty() {
            name.push('/');
        }
        name.push_str(part);
    }
    (!name.is_empty()).then_some(name)
}

/// Adds every file in [`files_dir`] to the blob store under a `files/` tag and
/// writes `files.json`. Idempotent: re-hosting the same content refreshes the
/// pin instead of duplicating the blob.
pub async fn host_files(store: &DataStore) -> anyhow::Result<Vec<HostedFile>> {
    let blobs: &Blobs = store.blobs().blobs();
    let mut hosted = Vec::new();

    for (name, path) in hosted_files(&files_dir())? {
        let size = fs::metadata(&path)?.len();

        let haf = blobs.add_path(&path).with_named_tag(tag(&name)).await?;
        info!(name, hash = %haf.hash, size, "hosting file over iroh");
        hosted.push(HostedFile {
            name,
            hash: haf.hash,
            size,
        });
    }

    sweep(store, &hosted).await?;

    let index = files_dir().join("files.json");
    fs::write(index, serde_json::to_string_pretty(&hosted)?)?;
    Ok(hosted)
}

/// Whether a tag is one this module owns, and so may be deleted by a sweep.
fn is_ours(name: &str) -> bool {
    name.starts_with(TAG_PREFIX)
}

/// Drops `files/` tags naming content the directory no longer holds, so a file
/// removed by the operator stops being served and its blob is reclaimed.
async fn sweep(store: &DataStore, hosted: &[HostedFile]) -> anyhow::Result<()> {
    let live = hosted.iter().map(|f| tag(&f.name)).collect::<Vec<_>>();

    let tags = store.blobs().tags();
    let mut stale = Vec::new();
    let mut listed = tags.list_prefix(TAG_PREFIX).await?;

    while let Some(info) = listed.next().await {
        let name = String::from_utf8_lossy(&info?.name.0).into_owned();
        if is_ours(&name) && !live.contains(&name) {
            stale.push(name);
        }
    }

    for name in stale {
        info!(name, "unhosting file removed from the files dir");
        tags.delete(&name).await?;
    }

    Ok(())
}

/// Prints the hosted hashes in the manifest shape the client consumes, so an
/// operator can paste them into `unavi-assets` and verify what the server
/// actually serves.
pub fn log_manifest(hosted: &[HostedFile]) {
    if hosted.is_empty() {
        warn!("no files hosted; drop files into {}", files_dir().display());
        return;
    }
    info!("hosted files (name = hash)");
    for file in hosted {
        info!("  {} = {}", file.name, file.hash);
    }
}

/// The pin tag a hosted file is stored under.
#[must_use]
pub fn tag(name: &str) -> String {
    format!("{TAG_PREFIX}{name}")
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{
        AtomicU32,
        Ordering,
    };

    use super::*;

    static NEXT: AtomicU32 = AtomicU32::new(0);

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "unavi-server-files-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn init_files_creates_the_dir_and_readme() {
        let dir = temp_dir();
        init_files(&dir).expect("init");

        assert!(dir.is_dir(), "the files dir is created");
        assert!(dir.join("README.md").is_file(), "the README is written");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn nested_files_are_hosted_under_their_relative_path() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join("model")).expect("mkdir");
        fs::write(dir.join("model/default.vrm"), b"vrm").expect("write");
        fs::write(dir.join("top.bin"), b"top").expect("write");

        let names = hosted_files(&dir)
            .expect("collect")
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();

        assert_eq!(names, ["model/default.vrm", "top.bin"], "sorted by name");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn hosting_metadata_is_not_itself_hosted() {
        let dir = temp_dir();
        init_files(&dir).expect("init");
        fs::write(dir.join("files.json"), b"[]").expect("write");
        fs::create_dir_all(dir.join("model")).expect("mkdir");
        fs::write(dir.join("model/README.md"), b"about the models").expect("write");

        let names = hosted_files(&dir)
            .expect("collect")
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            ["model/README.md"],
            "only the top-level README and index are excluded"
        );

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_not_followed() {
        let dir = temp_dir();
        let outside = temp_dir();
        fs::create_dir_all(&dir).expect("mkdir");
        fs::create_dir_all(&outside).expect("mkdir");
        fs::write(outside.join("secret.bin"), b"not yours").expect("write");
        std::os::unix::fs::symlink(outside.join("secret.bin"), dir.join("link.bin"))
            .expect("symlink");
        std::os::unix::fs::symlink(&outside, dir.join("link_dir")).expect("symlink");

        assert!(
            hosted_files(&dir).expect("collect").is_empty(),
            "a symlink never publishes a file from outside the hosted dir"
        );

        fs::remove_dir_all(&dir).expect("cleanup");
        fs::remove_dir_all(&outside).expect("cleanup");
    }

    #[test]
    fn init_files_keeps_existing_content() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join("operator.bin"), b"operator-owned").expect("write");

        init_files(&dir).expect("init");

        assert_eq!(
            fs::read(dir.join("operator.bin")).expect("read"),
            b"operator-owned",
            "an existing file is never touched"
        );

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn tag_namespace_pins() {
        assert_eq!(tag("model/default.vrm"), "files/model/default.vrm");
    }

    #[test]
    fn a_sweep_only_owns_its_own_namespace() {
        assert!(is_ours(&tag("model/default.vrm")));
        assert!(
            !is_ours("assets/model/default.vrm"),
            "a client manifest pin is another subsystem's"
        );
        assert!(
            !is_ours("cache/00000000000001700000000/abc"),
            "a dated cache root is another subsystem's"
        );
    }
}
