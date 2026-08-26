use iroh_docs::{
    Capability,
    NamespaceId,
    NamespaceSecret,
    api::Doc,
    protocol::Docs,
};

/// Makes a namespace available locally, importing it read-only if this node
/// does not already hold it.
///
/// `Docs::open` errors on an unknown namespace rather than returning
/// `Ok(None)`, so it cannot serve as the import path. Importing is idempotent:
/// merging a read capability into a write capability already held is a no-op,
/// not a downgrade.
pub async fn ensure_open(docs: &Docs, ns: NamespaceId) -> anyhow::Result<Doc> {
    let doc = docs.api().import_namespace(Capability::Read(ns)).await?;
    Ok(doc)
}

/// Opens a namespace this node holds the secret for, importing it if absent.
pub async fn ensure_writable(docs: &Docs, secret: NamespaceSecret) -> anyhow::Result<Doc> {
    let doc = docs
        .api()
        .import_namespace(Capability::Write(secret))
        .await?;
    Ok(doc)
}

/// Enrols a namespace in the sync set, so incoming requests for it are
/// answered. A namespace absent from the sync set rejects every incoming
/// request with `NotFound`; an empty peer list enrols without dialing anyone.
pub async fn serve(docs: &Docs, ns: NamespaceId) -> anyhow::Result<()> {
    ensure_open(docs, ns).await?.start_sync(Vec::new()).await?;
    Ok(())
}
