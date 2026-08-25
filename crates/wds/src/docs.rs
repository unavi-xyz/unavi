use iroh_docs::{
    Capability,
    NamespaceId,
    NamespaceSecret,
    api::Doc,
    protocol::Docs,
};

use crate::identity::store::{
    self,
    KeyStorage,
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

/// Opens the document this node keeps under `label`, minting it on first use.
///
/// The namespace is minted and its id recorded locally, never computed from the
/// identity key: a namespace id derived from a secret is one nobody else can
/// compute, and these documents exist to be read.
///
/// A recorded id whose capability the store no longer holds means the docs
/// store was lost. The document is unrecoverable either way, so a fresh one is
/// minted and the pointer replaced.
pub async fn well_known(docs: &Docs, storage: &KeyStorage, label: &str) -> anyhow::Result<Doc> {
    if let Some(ns) = store::load_namespace(storage, label)?
        && let Some(doc) = docs.api().open(ns).await?
    {
        return Ok(doc);
    }

    let doc = docs.api().create().await?;
    store::save_namespace(storage, label, doc.id())?;
    Ok(doc)
}

/// Opens a namespace this node holds the secret for, importing it if absent.
///
/// The path for a capability obtained elsewhere: received from a peer, or read
/// back from storage.
pub async fn ensure_writable(docs: &Docs, secret: NamespaceSecret) -> anyhow::Result<Doc> {
    let doc = docs
        .api()
        .import_namespace(Capability::Write(secret))
        .await?;
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
