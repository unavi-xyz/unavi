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
use unavi_local::LocalStorage;

use super::{
    BoxedBlobs,
    mem_store,
};

pub async fn init(
    storage: &LocalStorage,
    gc: Option<GcConfig>,
) -> anyhow::Result<(BoxedBlobs, DocsBuilder)> {
    let Some(dir) = storage.dir() else {
        let blobs: BoxedBlobs = Box::new(mem_store(gc));
        return Ok((blobs, Docs::memory()));
    };

    let root = dir.join("store");
    let blob_path = root.join("blob");
    let docs_path = root.join("docs");
    tokio::fs::create_dir_all(&blob_path).await?;
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
