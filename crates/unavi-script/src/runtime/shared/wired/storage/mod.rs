use anyhow::bail;
use async_channel::{
    Receiver,
    TryRecvError,
};
use bevy_iroh::doc::{
    DocGet,
    DocList,
};
use bytes::Bytes;
use iroh_docs::NamespaceId;
use unavi_identity::root_doc::root_doc;
use unavi_registry::follow::registries;
use unavi_util::async_commands::AsyncCommands;

use crate::runtime::shared::{
    Api,
    slot_map::SlotMap,
};

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

pub async fn get(api: &Api, _rep: u32, ns: Vec<u8>, key: String) -> anyhow::Result<u32> {
    let ns = namespace(&ns)?;
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .trigger(DocGet { ns, key, tx })
        .send()
        .await?;
    let mut storage = api.wired_storage.lock().await;
    Ok(storage
        .get_futures
        .insert(GetFutureRes { rx }, &api.quota)?)
}

pub async fn list(api: &Api, _rep: u32, ns: Vec<u8>, prefix: String) -> anyhow::Result<u32> {
    let ns = namespace(&ns)?;
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .trigger(DocList { ns, prefix, tx })
        .send()
        .await?;
    let mut storage = api.wired_storage.lock().await;
    Ok(storage
        .list_futures
        .insert(ListFutureRes { rx }, &api.quota)?)
}

pub fn root_doc_ns(_api: &Api, _rep: u32) -> anyhow::Result<Option<Vec<u8>>> {
    Ok(root_doc().map(|ns| ns.to_bytes().to_vec()))
}

pub fn registry_namespaces(_api: &Api, _rep: u32) -> anyhow::Result<Vec<Vec<u8>>> {
    Ok(registries()
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
