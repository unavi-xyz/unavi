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
/// so the obvious `match open { Some => .., None => import }` never reaches its
/// import arm — the `?` fails first. Importing is the idempotent operation:
/// iroh-docs imports then opens, and merging a read capability into a write one
/// already held is a no-op rather than a downgrade.
pub async fn ensure_open(docs: &Docs, ns: NamespaceId) -> anyhow::Result<Doc> {
    let doc = docs.api().import_namespace(Capability::Read(ns)).await?;
    Ok(doc)
}

/// Enrols a namespace in the sync set, so incoming requests for it are
/// answered.
///
/// Holding a namespace is not the same as serving it: one absent from the sync
/// set rejects every incoming request with `NotFound`. An empty peer list
/// enrols without dialing anyone, which is what an owner wants — readers come
/// to it.
pub async fn serve(docs: &Docs, ns: NamespaceId) -> anyhow::Result<()> {
    ensure_open(docs, ns).await?.start_sync(Vec::new()).await?;
    Ok(())
}
