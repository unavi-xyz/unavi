use anyhow::bail;
use async_channel::{
    Receiver,
    TryRecvError,
};
use bevy_wds::{
    blob::get::GetBlob,
    doc::{
        DocCreate,
        DocDelete,
        DocGet,
        DocList,
        DocSet,
    },
    registries,
    root_doc,
};
use blake3::Hash;
use bytes::Bytes;
use iroh_docs::NamespaceId;
use tracing::warn;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};

use crate::runtime::shared::{
    Api,
    slot_map::SlotMap,
};

pub struct WdsRes;

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

pub struct BlobFutureRes {
    _cancel: tokio::sync::oneshot::Sender<()>,
    rx:      Receiver<anyhow::Result<Bytes>>,
}

#[derive(Default)]
pub struct WiredWdsApi {
    wds_slots:    SlotMap<WdsRes>,
    get_futures:  SlotMap<GetFutureRes>,
    list_futures: SlotMap<ListFutureRes>,
    blob_futures: SlotMap<BlobFutureRes>,
}

fn namespace(bytes: &[u8]) -> anyhow::Result<NamespaceId> {
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("namespace id must be 32 bytes"))?;
    Ok(NamespaceId::from(&arr))
}

pub async fn get_wds(api: &Api) -> anyhow::Result<u32> {
    let mut wds = api.wired_wds.lock().await;
    Ok(wds.wds_slots.insert(WdsRes, &api.quota)?)
}

pub async fn create_doc(_api: &Api, _wds_rep: u32) -> anyhow::Result<Vec<u8>> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .trigger(DocCreate { tx })
        .send()
        .await?;
    let ns = rx
        .recv()
        .await?
        .ok_or_else(|| anyhow::anyhow!("create doc failed"))?;
    Ok(ns.to_bytes().to_vec())
}

pub async fn set(
    api: &Api,
    _wds_rep: u32,
    ns: Vec<u8>,
    key: String,
    value: Vec<u8>,
) -> anyhow::Result<()> {
    let ns = namespace(&ns)?;
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .trigger(DocSet {
            ns,
            key,
            value: value.into(),
            tx,
        })
        .send()
        .await?;
    if !rx.recv().await? {
        bail!("doc set failed (no write capability?)");
    }
    let _ = api;
    Ok(())
}

pub async fn delete(api: &Api, _wds_rep: u32, ns: Vec<u8>, key: String) -> anyhow::Result<()> {
    let ns = namespace(&ns)?;
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .trigger(DocDelete { ns, key, tx })
        .send()
        .await?;
    rx.recv().await?;
    let _ = api;
    Ok(())
}

pub async fn get(api: &Api, _wds_rep: u32, ns: Vec<u8>, key: String) -> anyhow::Result<u32> {
    let ns = namespace(&ns)?;
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .trigger(DocGet { ns, key, tx })
        .send()
        .await?;
    let mut wds = api.wired_wds.lock().await;
    Ok(wds.get_futures.insert(GetFutureRes { rx }, &api.quota)?)
}

pub async fn list(api: &Api, _wds_rep: u32, ns: Vec<u8>, prefix: String) -> anyhow::Result<u32> {
    let ns = namespace(&ns)?;
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .trigger(DocList { ns, prefix, tx })
        .send()
        .await?;
    let mut wds = api.wired_wds.lock().await;
    Ok(wds.list_futures.insert(ListFutureRes { rx }, &api.quota)?)
}

pub fn root_doc_ns(_api: &Api, _wds_rep: u32) -> anyhow::Result<Option<Vec<u8>>> {
    Ok(root_doc().map(|ns| ns.to_bytes().to_vec()))
}

pub fn registry_namespaces(_api: &Api, _wds_rep: u32) -> anyhow::Result<Vec<Vec<u8>>> {
    Ok(registries()
        .into_iter()
        .map(|ns| ns.to_bytes().to_vec())
        .collect())
}

pub async fn get_future_poll(
    api: &Api,
    rep: u32,
) -> anyhow::Result<Option<Result<Option<Vec<u8>>, ()>>> {
    let wds = api.wired_wds.lock().await;
    let Some(res) = wds.get_futures.get(rep) else {
        bail!("get future resource not found")
    };
    match res.rx.try_recv() {
        Ok(opt) => {
            drop(wds);
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
    let wds = api.wired_wds.lock().await;
    let Some(res) = wds.list_futures.get(rep) else {
        bail!("list future resource not found")
    };
    match res.rx.try_recv() {
        Ok(entries) => {
            drop(wds);
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
    api.wired_wds.lock().await.get_futures.remove(rep);
    Ok(())
}

pub async fn list_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_wds.lock().await.list_futures.remove(rep);
    Ok(())
}

pub async fn get_blob(api: &Api, _wds_rep: u32, blob_id: Vec<u8>) -> anyhow::Result<u32> {
    let hash = Hash::from_slice(&blob_id)?;
    let (tx, rx) = async_channel::bounded(1);
    let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();
    spawn_async_task(async move {
        tokio::select! {
            _ = &mut cancel_rx => {}
            result = fetch_blob(hash) => {
                let _ = tx.try_send(result);
            }
        }
    });
    Ok(api.wired_wds.lock().await.blob_futures.insert(
        BlobFutureRes {
            _cancel: cancel_tx,
            rx,
        },
        &api.quota,
    )?)
}

async fn fetch_blob(hash: Hash) -> anyhow::Result<Bytes> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .trigger(GetBlob {
            hash,
            cancel: None,
            tx,
        })
        .send()
        .await?;
    Ok(rx.recv().await??)
}

pub async fn blob_future_poll(api: &Api, rep: u32) -> anyhow::Result<Option<Result<Vec<u8>, ()>>> {
    let wds = api.wired_wds.lock().await;
    let Some(res) = wds.blob_futures.get(rep) else {
        bail!("blob future resource not found")
    };
    match res.rx.try_recv() {
        Ok(Ok(bytes)) => {
            drop(wds);
            Ok(Some(Ok(bytes.to_vec())))
        }
        Ok(Err(err)) => {
            drop(wds);
            warn!(?err, "blob_future_poll: fetch failed");
            Ok(Some(Err(())))
        }
        Err(TryRecvError::Empty) => Ok(None),
        Err(TryRecvError::Closed) => Ok(Some(Err(()))),
    }
}

pub async fn blob_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_wds.lock().await.blob_futures.remove(rep);
    Ok(())
}

pub async fn wds_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_wds.lock().await.wds_slots.remove(rep);
    Ok(())
}
