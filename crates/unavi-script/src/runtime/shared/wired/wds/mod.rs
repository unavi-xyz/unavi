use async_channel::Receiver;
use bevy_wds::record::{query::QueryRecord, read::ReadRecord};
use blake3::Hash;
use loro::{ExportMode, LoroDoc};
use unavi_util::async_commands::AsyncCommands;

use crate::runtime::shared::{Api, slot_map::SlotMap};

pub struct WdsRes;

pub struct QueryFutureRes;

pub struct ReadFutureRes;

#[derive(Default)]
pub struct WiredWdsApi {
    wds_slots: SlotMap<WdsRes>,
    query_futures: SlotMap<Receiver<Vec<Hash>>>,
    read_futures: SlotMap<(Hash, Receiver<LoroDoc>)>,
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
    let (mut event, rx, _cancel) = QueryRecord::new();

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
    Ok(wds.query_futures.insert(rx))
}

pub fn read(api: &Api, _wds_rep: u32, record_id: Vec<u8>) -> anyhow::Result<u32> {
    let id = Hash::from_slice(&record_id)?;
    let (event, rx, _cancel) = ReadRecord::new(id);

    AsyncCommands::default().trigger(event).try_send()?;

    let mut wds = api.wired_wds.try_lock()?;
    Ok(wds.read_futures.insert((id, rx)))
}

pub fn query_future_poll(api: &Api, rep: u32) -> anyhow::Result<Option<Result<Vec<Vec<u8>>, ()>>> {
    let wds = api.wired_wds.try_lock()?;
    let Some(rx) = wds.query_futures.get(rep) else {
        return Ok(Some(Err(())));
    };
    match rx.try_recv() {
        Ok(hashes) => Ok(Some(Ok(hashes
            .into_iter()
            .map(|h| h.as_bytes().to_vec())
            .collect()))),
        Err(async_channel::TryRecvError::Empty) => Ok(None),
        Err(async_channel::TryRecvError::Closed) => Ok(Some(Err(()))),
    }
}

pub fn read_future_poll(api: &Api, rep: u32) -> anyhow::Result<Option<Result<WdsRecord, ()>>> {
    let wds = api.wired_wds.try_lock()?;
    let Some((id, rx)) = wds.read_futures.get(rep) else {
        return Ok(Some(Err(())));
    };
    match rx.try_recv() {
        Ok(doc) => Ok(Some(Ok(WdsRecord {
            id: id.as_bytes().to_vec(),
            creator: String::new(),
            schemas: Vec::new(),
            containers: vec![("data".to_string(), doc.export(ExportMode::Snapshot)?)],
        }))),
        Err(async_channel::TryRecvError::Empty) => Ok(None),
        Err(async_channel::TryRecvError::Closed) => Ok(Some(Err(()))),
    }
}

pub fn query_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    let mut wds = api.wired_wds.try_lock()?;
    wds.query_futures.remove(rep);
    Ok(())
}

pub fn read_future_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    let mut wds = api.wired_wds.try_lock()?;
    wds.read_futures.remove(rep);
    Ok(())
}

pub fn wds_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    let mut wds = api.wired_wds.try_lock()?;
    wds.wds_slots.remove(rep);
    Ok(())
}
