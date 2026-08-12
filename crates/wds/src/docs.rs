use iroh_docs::{
    Capability,
    NamespaceId,
    api::Doc,
    protocol::Docs,
};

/// Makes a namespace available locally, importing it read-only if this node
/// does not already hold it.
///
/// `Docs::open` returns `Err` for an unknown namespace rather than `Ok(None)`,
/// so `open` alone cannot serve as the import path. Importing is idempotent:
/// merging a read capability into a write one already held is a no-op rather
/// than a downgrade.
pub async fn ensure_open(docs: &Docs, ns: NamespaceId) -> anyhow::Result<Doc> {
    let doc = docs.api().import_namespace(Capability::Read(ns)).await?;
    Ok(doc)
}

/// Enrols a namespace in the sync set, so incoming requests for it are
/// answered.
///
/// Holding a namespace is not the same as serving it: one absent from the sync
/// set rejects every incoming request with `NotFound`. An empty peer list
/// enrols without dialing anyone.
pub async fn serve(docs: &Docs, ns: NamespaceId) -> anyhow::Result<()> {
    ensure_open(docs, ns).await?.start_sync(Vec::new()).await?;
    Ok(())
}
