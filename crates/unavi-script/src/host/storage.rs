//! `unavi:host/node-storage`: reads of this node's root document and the
//! registry views it follows. No other document is readable, and a read
//! never imports one.

use std::pin::pin;

use async_channel::{
    Receiver,
    TryRecvError,
};
use bevy_async::task;
use bevy_iroh::store::DataStore;
use hsd::id::DocId;
use iroh_docs::NamespaceId;
use n0_future::StreamExt;
use parking_lot::Mutex;
use unavi_policy::permissions::HostApi;
use unavi_store::{
    Document,
    MAX_ENTRY_BYTES,
    Tombstones,
};

use crate::{
    error::ScriptError,
    host::ScriptHost,
};

/// Most entries one listing returns.
pub const MAX_LIST_ENTRIES: u32 = 256;

pub type Entries = Vec<(String, Vec<u8>)>;

/// A read that resolves later. Once resolved, every poll answers the same.
pub struct Pending<T> {
    rx:       Receiver<Result<T, ScriptError>>,
    resolved: Mutex<Option<Result<T, ScriptError>>>,
}

pub type PendingValue = Pending<Option<Vec<u8>>>;
pub type PendingEntries = Pending<Entries>;

impl<T: Clone> Pending<T> {
    fn spawn(work: impl Future<Output = Result<T, ScriptError>> + Send + 'static) -> Self
    where
        T: Send + 'static,
    {
        let (tx, rx) = async_channel::bounded(1);
        task::spawn(async move {
            let _ = tx.send(work.await).await;
        });
        Self {
            rx,
            resolved: Mutex::new(None),
        }
    }

    /// `None` until the read finishes.
    pub fn poll(&self) -> Option<Result<T, ScriptError>> {
        let mut resolved = self.resolved.lock();
        if resolved.is_none() {
            *resolved = match self.rx.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Closed) => {
                    Some(Err(ScriptError::Internal("the read was dropped".into())))
                }
            };
        }
        resolved.clone()
    }
}

/// The root document and the followed registry views, which is all the shell
/// reads.
fn readable(host: &ScriptHost, ns: NamespaceId) -> bool {
    host.root_doc == Some(ns) || host.view.identity().followed.is_view(ns)
}

fn require(host: &ScriptHost) -> Result<(), ScriptError> {
    host.require(HostApi::Storage)
        .map_err(|_| ScriptError::Forbidden)
}

pub fn root_document(host: &ScriptHost) -> Result<Option<DocId>, ScriptError> {
    require(host)?;
    Ok(host.root_doc.map(|ns| DocId(*ns.as_bytes())))
}

pub fn registries(host: &ScriptHost) -> Result<Vec<DocId>, ScriptError> {
    require(host)?;
    Ok(host
        .view
        .identity()
        .followed
        .views()
        .into_iter()
        .map(|ns| DocId(*ns.as_bytes()))
        .collect())
}

async fn held(host: &ScriptHost, doc: DocId) -> Result<Document, ScriptError> {
    require(host)?;
    let ns = NamespaceId::from(&doc.0);
    if !readable(host, ns) {
        return Err(ScriptError::NotFound);
    }
    let store = host
        .world_call(|world| world.get_resource::<DataStore>().map(|s| s.0.clone()))
        .await?
        .ok_or(ScriptError::NotFound)?;
    store
        .held(ns)
        .await
        .map_err(ScriptError::internal)?
        .ok_or(ScriptError::NotFound)
}

pub async fn get(host: &mut ScriptHost, doc: DocId, key: String) -> Result<u32, ScriptError> {
    let doc = held(host, doc).await?;
    let pending = Pending::spawn(async move {
        Ok(doc
            .get(key)
            .await
            .map_err(ScriptError::internal)?
            .map(|value| value.to_vec()))
    });
    Ok(host.pending_values.insert(pending, &host.quota)?)
}

/// Entries whose content has not downloaded, or is oversized, are left out.
pub async fn list_entries(
    host: &mut ScriptHost,
    doc: DocId,
    prefix: String,
    limit: u32,
) -> Result<u32, ScriptError> {
    let doc = held(host, doc).await?;
    let limit = limit.min(MAX_LIST_ENTRIES) as usize;
    let pending = Pending::spawn(async move {
        list(&doc, &prefix, limit)
            .await
            .map_err(ScriptError::internal)
    });
    Ok(host.pending_entries.insert(pending, &host.quota)?)
}

async fn list(doc: &Document, prefix: &str, limit: usize) -> unavi_store::Result<Entries> {
    let mut entries = pin!(doc.entries(prefix, Tombstones::Exclude).await?);
    let mut out = Vec::new();
    while out.len() < limit
        && let Some(entry) = entries.next().await
    {
        let entry = entry?;
        // No key this workspace writes is anything but UTF-8, so one that does
        // not decode names nothing a caller could have asked for.
        let Ok(key) = String::from_utf8(entry.key().to_vec()) else {
            continue;
        };
        if entry.content_len() > MAX_ENTRY_BYTES {
            continue;
        }
        if let Some(value) = doc.value(&entry).await? {
            out.push((key, value.to_vec()));
        }
    }
    Ok(out)
}
