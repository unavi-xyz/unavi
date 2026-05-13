use anyhow::bail;
use async_channel::{Receiver, TryRecvError};
use bevy_wds::record::{query::QueryRecord, read::ReadRecord};
use blake3::Hash;
use loro::{ExportMode, LoroDoc};
use tokio::sync::oneshot::Sender;
use tracing::warn;
use unavi_util::async_commands::AsyncCommands;

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

#[derive(Default)]
pub struct WiredWdsApi {
    wds_slots: SlotMap<WdsRes>,
    query_futures: SlotMap<QueryFutureRes>,
    read_futures: SlotMap<ReadFutureRes>,
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

pub fn get_wds(api: &Api) -> anyhow::Result<u32> {
    let mut wds = api.wired_wds.try_lock()?;
    Ok(wds.wds_slots.insert(WdsRes))
}

pub fn query(api: &Api, _wds_rep: u32, filter: Option<QueryFilter>) -> anyhow::Result<u32> {
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

    let mut wds = api.wired_wds.try_lock()?;
    Ok(wds.query_futures.insert(QueryFutureRes {
        _cancel: cancel,
        rx,
    }))
}

pub fn read(api: &Api, _wds_rep: u32, record_id: Vec<u8>) -> anyhow::Result<u32> {
    let id = Hash::from_slice(&record_id)?;
    let (event, rx, cancel) = ReadRecord::new(id);

    AsyncCommands::default().trigger(event).try_send()?;

    let mut wds = api.wired_wds.try_lock()?;
    Ok(wds.read_futures.insert(ReadFutureRes {
        _cancel: cancel,
        id,
        rx,
    }))
}

pub fn query_future_poll(api: &Api, rep: u32) -> anyhow::Result<Option<Result<Vec<Vec<u8>>, ()>>> {
    let wds = api.wired_wds.try_lock()?;
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

pub fn read_future_poll(api: &Api, rep: u32) -> anyhow::Result<Option<Result<WdsRecord, ()>>> {
    let wds = api.wired_wds.try_lock()?;
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

pub fn query_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_wds.try_lock()?.query_futures.remove(rep);
    Ok(())
}

pub fn read_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_wds.try_lock()?.read_futures.remove(rep);
    Ok(())
}

pub fn wds_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_wds.try_lock()?.wds_slots.remove(rep);
    Ok(())
}
