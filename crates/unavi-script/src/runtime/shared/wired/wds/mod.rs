use anyhow::bail;
use async_channel::{Receiver, TryRecvError};
use bevy_wds::{
    blob::get::GetBlob,
    record::{query::QueryRecord, read::ReadRecord},
};
use blake3::Hash;
use bytes::Bytes;
use loro::{ExportMode, LoroDoc};
use tokio::sync::oneshot::{self, Sender};
use tracing::warn;
use unavi_util::{async_commands::AsyncCommands, async_task::spawn_async_task};

use crate::runtime::shared::{Api, slot_map::SlotMap};

pub struct WdsRes;

pub struct QueryFutureRes {
    _cancel: Sender<()>,
    rx: Receiver<Vec<Hash>>,
}

pub struct ReadFutureRes {
    _cancel: Sender<()>,
    id: Hash,
    rx: Receiver<LoroDoc>,
}

pub struct BlobFutureRes {
    _cancel: Sender<()>,
    rx: Receiver<anyhow::Result<Bytes>>,
}

#[derive(Default)]
pub struct WiredWdsApi {
    wds_slots: SlotMap<WdsRes>,
    query_futures: SlotMap<QueryFutureRes>,
    read_futures: SlotMap<ReadFutureRes>,
    blob_futures: SlotMap<BlobFutureRes>,
}

pub struct QueryFilter {
    pub creator: Option<String>,
    pub schemas: Option<Vec<Vec<u8>>>,
}

pub struct WdsRecord {
    pub id: Vec<u8>,
    pub creator: String,
    pub schemas: Vec<Vec<u8>>,
    pub containers: Vec<(String, Vec<u8>)>,
}

pub async fn get_wds(api: &Api) -> anyhow::Result<u32> {
    let mut wds = api.wired_wds.lock().await;
    Ok(wds.wds_slots.insert(WdsRes))
}

pub async fn query(api: &Api, _wds_rep: u32, filter: Option<QueryFilter>) -> anyhow::Result<u32> {
    let (mut event, rx, cancel) = QueryRecord::new();

    if let Some(f) = filter {
        event.creator = f.creator;
        event.schemas = f
            .schemas
            .unwrap_or_default()
            .into_iter()
            .filter_map(|b| Hash::from_slice(&b).ok())
            .collect();
    }

    AsyncCommands::default().trigger(event).try_send()?;

    let mut wds = api.wired_wds.lock().await;
    Ok(wds.query_futures.insert(QueryFutureRes {
        _cancel: cancel,
        rx,
    }))
}

pub async fn read(api: &Api, _wds_rep: u32, record_id: Vec<u8>) -> anyhow::Result<u32> {
    let id = Hash::from_slice(&record_id)?;
    let (event, rx, cancel) = ReadRecord::new(id);

    AsyncCommands::default().trigger(event).try_send()?;

    let mut wds = api.wired_wds.lock().await;
    Ok(wds.read_futures.insert(ReadFutureRes {
        _cancel: cancel,
        id,
        rx,
    }))
}

pub async fn query_future_poll(
    api: &Api,
    rep: u32,
) -> anyhow::Result<Option<Result<Vec<Vec<u8>>, ()>>> {
    let wds = api.wired_wds.lock().await;
    let Some(res) = wds.query_futures.get(rep) else {
        bail!("query future resource not found")
    };
    match res.rx.try_recv() {
        Ok(hashes) => {
            drop(wds);
            Ok(Some(Ok(hashes
                .into_iter()
                .map(|h| h.as_bytes().to_vec())
                .collect())))
        }
        Err(TryRecvError::Empty) => Ok(None),
        Err(TryRecvError::Closed) => {
            warn!("query_future_poll: channel closed");
            Ok(Some(Err(())))
        }
    }
}

pub async fn read_future_poll(
    api: &Api,
    rep: u32,
) -> anyhow::Result<Option<Result<WdsRecord, ()>>> {
    let wds = api.wired_wds.lock().await;
    let Some(res) = wds.read_futures.get(rep) else {
        return Ok(Some(Err(())));
    };
    match res.rx.try_recv() {
        Ok(doc) => {
            let id = res.id.as_bytes().to_vec();
            drop(wds);
            Ok(Some(Ok(WdsRecord {
                id,
                creator: String::new(),
                schemas: Vec::new(),
                containers: vec![("data".to_string(), doc.export(ExportMode::Snapshot)?)],
            })))
        }
        Err(TryRecvError::Empty) => Ok(None),
        Err(TryRecvError::Closed) => Ok(Some(Err(()))),
    }
}

pub async fn query_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_wds.lock().await.query_futures.remove(rep);
    Ok(())
}

pub async fn read_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_wds.lock().await.read_futures.remove(rep);
    Ok(())
}

pub async fn get_blob(api: &Api, _wds_rep: u32, blob_id: Vec<u8>) -> anyhow::Result<u32> {
    let hash = Hash::from_slice(&blob_id)?;
    let (tx, rx) = async_channel::bounded(1);
    let (cancel_tx, mut cancel_rx) = oneshot::channel();
    spawn_async_task(async move {
        tokio::select! {
            _ = &mut cancel_rx => {}
            result = fetch_blob(hash) => {
                let _ = tx.try_send(result);
            }
        }
    });
    Ok(api
        .wired_wds
        .lock()
        .await
        .blob_futures
        .insert(BlobFutureRes {
            _cancel: cancel_tx,
            rx,
        }))
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
    Ok(rx.recv().await?)
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
