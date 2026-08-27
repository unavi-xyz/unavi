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

use super::{
    BoxedBlobs,
    mem_store,
};
use crate::local::Storage;

/// Subdirectory of a node's storage the blob and document stores live under.
const STORE_DIR: &str = "store";

pub async fn init(
    storage: &Storage,
    gc: Option<GcConfig>,
) -> anyhow::Result<(BoxedBlobs, DocsBuilder)> {
    let Some(dir) = storage.dir() else {
        let blobs: BoxedBlobs = Box::new(mem_store(gc));
        return Ok((blobs, Docs::memory()));
    };

    let root = dir.join(STORE_DIR);
    let blob_path = root.join("blob");
    let docs_path = root.join("docs");
    tokio::fs::create_dir_all(&blob_path).await?;
    // `Docs::persistent` opens its directory rather than creating it.
    tokio::fs::create_dir_all(&docs_path).await?;

    let blobs: BoxedBlobs = Box::new(
        FsStore::load_with_opts(
            blob_path.join("blobs.db"),
            Options {
                gc,
                ..Options::new(&blob_path)
            },
        )
        .await?,
    );

    Ok((blobs, Docs::persistent(docs_path)))
}
