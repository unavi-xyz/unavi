use iroh_blobs::store::GcConfig;
use iroh_docs::protocol::{
    Builder as DocsBuilder,
    Docs,
};

use super::{
    BoxedBlobs,
    mem_store,
};

/// The blob and document stores are always in memory on wasm — the storage
/// only persists recorded keys in browser local storage.
pub fn init(gc: Option<GcConfig>) -> anyhow::Result<(BoxedBlobs, DocsBuilder)> {
    let blobs: BoxedBlobs = Box::new(mem_store(gc));
    Ok((blobs, Docs::memory()))
}
