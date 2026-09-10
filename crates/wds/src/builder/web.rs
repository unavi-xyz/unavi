use iroh_blobs::store::GcConfig;
use iroh_docs::protocol::{
    Builder as DocsBuilder,
    Docs,
};

use super::{
    BoxedBlobs,
    mem_store,
};

pub fn init(gc: Option<GcConfig>) -> anyhow::Result<(BoxedBlobs, DocsBuilder)> {
    // TODO replace in-memory storage with IndexDB backend
    let blobs: BoxedBlobs = Box::new(mem_store(gc));
    Ok((blobs, Docs::memory()))
}
