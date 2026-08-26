//! Namespace management: opening, serving, and recording document ids.

use std::str::FromStr;

use iroh_docs::{
    Capability,
    NamespaceId,
    api::Doc,
    protocol::Docs,
};

use crate::local::Storage;

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

/// Enrols a namespace in the sync set, so incoming requests for it are
/// answered.
///
/// A namespace absent from the sync set rejects every incoming request with
/// `NotFound`; an empty peer list enrols without dialing anyone.
pub async fn serve(docs: &Docs, ns: NamespaceId) -> anyhow::Result<()> {
    ensure_open(docs, ns).await?.start_sync(Vec::new()).await?;
    Ok(())
}

/// Opens the namespace `storage` records at `key`, minting and recording one
/// on first use.
///
/// The id outlives the process, so a restart reopens the document peers already
/// reference. A recorded id whose capability the docs store no longer holds
/// means that store was lost; the document is unrecoverable either way, so a
/// fresh one is minted and the record replaced.
///
/// [`Storage::Ephemeral`] mints fresh every run and leaves nothing behind.
pub async fn open_or_mint(
    docs: &Docs,
    storage: &Storage,
    key: &str,
) -> anyhow::Result<NamespaceId> {
    if let Some(ns) = recorded(storage, key)
        && held(docs, ns).await
    {
        return Ok(ns);
    }

    let ns = docs.api().create().await?.id();
    storage.write(key, &ns.to_string())?;
    Ok(ns)
}

/// `Docs::open` errors on a namespace this node does not hold rather than
/// returning `Ok(None)`, so both shapes of absence have to read alike; a
/// propagated error would leave the node with no document at all.
async fn held(docs: &Docs, ns: NamespaceId) -> bool {
    match docs.api().open(ns).await {
        Ok(doc) => doc.is_some(),
        Err(err) => {
            tracing::warn!(%ns, ?err, "recorded namespace is unreadable; minting a replacement");
            false
        }
    }
}

/// [`open_or_mint`], plus enrolment in the sync set so peers can read it.
pub async fn serve_or_mint(
    docs: &Docs,
    storage: &Storage,
    key: &str,
) -> anyhow::Result<NamespaceId> {
    let ns = open_or_mint(docs, storage, key).await?;
    serve(docs, ns).await?;
    Ok(ns)
}

fn recorded(storage: &Storage, key: &str) -> Option<NamespaceId> {
    NamespaceId::from_str(storage.read(key)?.trim()).ok()
}
