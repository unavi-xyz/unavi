use iroh_blobs::store::GcConfig;
use iroh_docs::protocol::{
    Builder as DocsBuilder,
    Docs,
};

use super::{
    BoxedBlobs,
    mem_store,
};
use crate::local::Storage;

pub fn init(storage: &Storage, gc: Option<GcConfig>) -> anyhow::Result<(BoxedBlobs, DocsBuilder)> {
    anyhow::ensure!(
        storage.dir().is_none(),
        "file storage is not supported on wasm"
    );
    let blobs: BoxedBlobs = Box::new(mem_store(gc));
    Ok((blobs, Docs::memory()))
}
