//! `wired:storage`: reads of this node's root document and the view docs of
//! the registries it follows. No other namespace is readable, and a read never
//! imports one.

use std::pin::pin;

use anyhow::bail;
use async_channel::{
    Receiver,
    TryRecvError,
};
use bevy::prelude::*;
use bevy_async::task;
use bevy_iroh::store::DataStore;
use bytes::Bytes;
use iroh_docs::NamespaceId;
use n0_future::StreamExt;
use unavi_store::{
    Document,
    MAX_ENTRY_BYTES,
    Tombstones,
};

use crate::runtime::shared::{
    Api,
    slot_map::SlotMap,
};

/// Most entries one `list` returns.
const MAX_LIST_ENTRIES: usize = 256;

pub struct StorageRes;

/// A single key/value entry, as returned to guests.
pub struct EntryOut {
    pub key:   String,
    pub value: Vec<u8>,
}

pub struct GetFutureRes {
    rx: Receiver<Option<Bytes>>,
}

pub struct ListFutureRes {
    rx: Receiver<Vec<(String, Bytes)>>,
}

#[derive(Default)]
pub struct WiredStorageApi {
    storage_slots: SlotMap<StorageRes>,
    get_futures:   SlotMap<GetFutureRes>,
    list_futures:  SlotMap<ListFutureRes>,
}

fn namespace(bytes: &[u8]) -> anyhow::Result<NamespaceId> {
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("namespace id must be 32 bytes"))?;
    Ok(NamespaceId::from(&arr))
}

pub async fn get_storage(api: &Api) -> anyhow::Result<u32> {
    let mut storage = api.wired_storage.lock().await;
    Ok(storage.storage_slots.insert(StorageRes, &api.quota)?)
}

/// The root document and followed registry views, which is all the shell
/// reads. The script's own documents are `wired:scene`'s.
fn readable(api: &Api, ns: NamespaceId) -> bool {
    api.root_doc == Some(ns) || api.view.identity().followed.is_view(ns)
}

/// The store's document for `ns`, if `ns` is readable and held.
async fn held(api: &Api, ns: NamespaceId) -> anyhow::Result<Option<Document>> {
    if !readable(api, ns) {
        return Ok(None);
    }

    let (tx, rx) = async_channel::bounded(1);
    api.async_world
        .commands()
        .push(move |world: &mut World| {
            let Some(store) = world
                .get_resource::<DataStore>()
                .map(|store| store.0.clone())
            else {
                return;
            };
            task::spawn(async move {
                match store.held(ns).await {
                    Ok(doc) => {
                        tx.send(doc).await.ok();
                    }
                    Err(err) => debug!(%ns, ?err, "storage read failed"),
                }
            });
        })
        .send()
        .await?;

    Ok(rx.recv().await.ok().flatten())
}

/// An unreadable namespace, or one not held, resolves as a failed future.
pub async fn get(api: &Api, _rep: u32, ns: Vec<u8>, key: String) -> anyhow::Result<u32> {
    let ns = namespace(&ns)?;
    let (tx, rx) = async_channel::bounded(1);
    if let Some(doc) = held(api, ns).await? {
        task::spawn(async move {
            match doc.get(key).await {
                Ok(value) => {
                    tx.send(value).await.ok();
                }
                Err(err) => debug!(%ns, ?err, "storage get failed"),
            }
        });
    }
    let mut storage = api.wired_storage.lock().await;
    Ok(storage
        .get_futures
        .insert(GetFutureRes { rx }, &api.quota)?)
}

/// At most [`MAX_LIST_ENTRIES`], in key order. Entries whose content has not
/// downloaded are left out.
pub async fn list(api: &Api, _rep: u32, ns: Vec<u8>, prefix: String) -> anyhow::Result<u32> {
    let ns = namespace(&ns)?;
    let (tx, rx) = async_channel::bounded(1);
    if let Some(doc) = held(api, ns).await? {
        task::spawn(async move {
            match list_entries(&doc, &prefix).await {
                Ok(entries) => {
                    tx.send(entries).await.ok();
                }
                Err(err) => debug!(%ns, ?err, "storage list failed"),
            }
        });
    }
    let mut storage = api.wired_storage.lock().await;
    Ok(storage
        .list_futures
        .insert(ListFutureRes { rx }, &api.quota)?)
}

async fn list_entries(doc: &Document, prefix: &str) -> unavi_store::Result<Vec<(String, Bytes)>> {
    let mut entries = pin!(doc.entries(prefix, Tombstones::Exclude).await?);
    let mut out = Vec::new();
    while out.len() < MAX_LIST_ENTRIES
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
            out.push((key, value));
        }
    }
    Ok(out)
}

pub fn root_doc_ns(api: &Api, _rep: u32) -> anyhow::Result<Option<Vec<u8>>> {
    Ok(api.root_doc.map(|ns| ns.to_bytes().to_vec()))
}

pub fn registry_namespaces(api: &Api, _rep: u32) -> anyhow::Result<Vec<Vec<u8>>> {
    Ok(api
        .view
        .identity()
        .followed
        .views()
        .into_iter()
        .map(|ns| ns.to_bytes().to_vec())
        .collect())
}

pub async fn get_future_poll(
    api: &Api,
    rep: u32,
) -> anyhow::Result<Option<Result<Option<Vec<u8>>, ()>>> {
    let storage = api.wired_storage.lock().await;
    let Some(res) = storage.get_futures.get(rep) else {
        bail!("get future resource not found")
    };
    match res.rx.try_recv() {
        Ok(opt) => {
            drop(storage);
            Ok(Some(Ok(opt.map(|b| b.to_vec()))))
        }
        Err(TryRecvError::Empty) => Ok(None),
        Err(TryRecvError::Closed) => Ok(Some(Err(()))),
    }
}

pub async fn list_future_poll(
    api: &Api,
    rep: u32,
) -> anyhow::Result<Option<Result<Vec<EntryOut>, ()>>> {
    let storage = api.wired_storage.lock().await;
    let Some(res) = storage.list_futures.get(rep) else {
        bail!("list future resource not found")
    };
    match res.rx.try_recv() {
        Ok(entries) => {
            drop(storage);
            Ok(Some(Ok(entries
                .into_iter()
                .map(|(key, value)| EntryOut {
                    key,
                    value: value.to_vec(),
                })
                .collect())))
        }
        Err(TryRecvError::Empty) => Ok(None),
        Err(TryRecvError::Closed) => Ok(Some(Err(()))),
    }
}

pub async fn get_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_storage.lock().await.get_futures.remove(rep);
    Ok(())
}

pub async fn list_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_storage.lock().await.list_futures.remove(rep);
    Ok(())
}

pub async fn storage_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_storage.lock().await.storage_slots.remove(rep);
    Ok(())
}
