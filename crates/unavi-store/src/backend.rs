//! Where blobs and documents live: on disk under the storage root, or in
//! memory when there is none.

use std::path::PathBuf;

use iroh_blobs::{
    api::Store as BlobStore,
    store::{
        GcConfig,
        mem::{
            MemStore,
            Options as MemOptions,
        },
    },
};
use iroh_docs::protocol::{
    Builder as DocsBuilder,
    Docs,
};
use unavi_local::DeviceStorage;

/// Keeps the concrete blob store, and with it its actor, alive.
#[derive(Debug)]
pub enum Backend {
    Mem(MemStore),
    #[cfg(not(target_family = "wasm"))]
    Fs(iroh_blobs::store::fs::FsStore),
}

impl Backend {
    pub fn store(&self) -> &BlobStore {
        match self {
            Self::Mem(store) => store,
            #[cfg(not(target_family = "wasm"))]
            Self::Fs(store) => store,
        }
    }
}

/// On wasm everything is in memory, so a reload starts empty.
// TODO: persist on wasm through IndexedDB.
pub async fn open(
    storage: &DeviceStorage,
    gc: Option<GcConfig>,
) -> anyhow::Result<(Backend, DocsBuilder)> {
    match storage.dir() {
        #[cfg(not(target_family = "wasm"))]
        Some(dir) => fs::open(&dir.join(STORE_DIR), gc).await,
        _ => {
            let blobs = MemStore::new_with_opts(MemOptions { gc_config: gc });
            Ok((Backend::Mem(blobs), Docs::memory()))
        }
    }
}

const STORE_DIR: &str = "store";

/// Moves a store that failed to load with `err` aside, returning where to. The
/// identity is kept outside it, so starting empty loses cached content and
/// the documents recorded by name, which are minted again.
///
/// `None` when there is nothing on disk to move, or when the store is only
/// locked by another process, which moving would take out from under it.
pub fn quarantine(storage: &DeviceStorage, err: &anyhow::Error) -> anyhow::Result<Option<PathBuf>> {
    let Some(dir) = storage.dir() else {
        return Ok(None);
    };
    if cfg!(target_family = "wasm") || format!("{err:#}").contains("already open") {
        return Ok(None);
    }

    let store = dir.join(STORE_DIR);
    if !store.exists() {
        return Ok(None);
    }

    let stamp = n0_future::time::SystemTime::now()
        .duration_since(n0_future::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let moved = dir.join(format!("{STORE_DIR}.corrupt-{stamp}"));
    std::fs::rename(&store, &moved)?;
    Ok(Some(moved))
}

#[cfg(not(target_family = "wasm"))]
mod fs {
    use std::path::Path;

    use iroh_blobs::store::{
        GcConfig,
        fs::{
            FsStore,
            options::Options,
        },
    };
    use iroh_docs::protocol::{
        Builder as DocsBuilder,
        Docs,
    };

    use super::Backend;

    pub async fn open(root: &Path, gc: Option<GcConfig>) -> anyhow::Result<(Backend, DocsBuilder)> {
        let blob_path = root.join("blob");
        let docs_path = root.join("docs");
        tokio::fs::create_dir_all(&blob_path).await?;
        tokio::fs::create_dir_all(&docs_path).await?;

        let blobs = FsStore::load_with_opts(
            blob_path.join("blobs.db"),
            Options {
                gc,
                ..Options::new(&blob_path)
            },
        )
        .await?;

        Ok((Backend::Fs(blobs), Docs::persistent(docs_path)))
    }
}
