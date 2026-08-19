use bevy::prelude::*;
use hsd::id::DocId;
use iroh_docs::NamespaceId;
use unavi_policy::check::{
    read as check_read,
    space_of,
    write as check_write,
};
use unavi_space::state::{
    entities,
    replicas,
};

use crate::{
    error::ScriptError,
    runtime::shared::{
        Api,
        slot_map::SlotMap,
    },
};

#[derive(Clone, Copy)]
pub struct KvRes {
    pub space: DocId,
    pub doc:   DocId,
}

/// Replicas key by 32 opaque bytes; a document id is one. Keying by document id
/// is what lets a prefab instance — which has an id but no namespace — hold
/// shared state at all.
fn ns(id: DocId) -> NamespaceId {
    NamespaceId::from(&id.0)
}

#[derive(Default)]
pub struct WiredKvApi {
    kv_slots: SlotMap<KvRes>,
}

pub async fn self_kv(api: &Api) -> anyhow::Result<u32> {
    let Some(space) = space_of(api.doc_id) else {
        anyhow::bail!("script's host document is not in a tracked space");
    };
    let mut slots = api.wired_kv.lock().await;
    Ok(slots.kv_slots.insert(
        KvRes {
            space,
            doc: api.doc_id,
        },
        &api.quota,
    )?)
}

pub async fn get_kv(api: &Api, doc_id: Vec<u8>) -> anyhow::Result<Option<u32>> {
    let Some(space) = space_of(api.doc_id) else {
        return Ok(None);
    };
    let Ok(bytes) = <[u8; 32]>::try_from(doc_id.as_slice()) else {
        return Ok(None);
    };
    let doc = DocId(bytes);

    if !replicas::has_doc(ns(space), ns(doc)) {
        return Ok(None);
    }

    if check_read(api.doc_id, doc).is_err() {
        return Ok(None);
    }

    let mut slots = api.wired_kv.lock().await;
    Ok(Some(
        slots.kv_slots.insert(KvRes { space, doc }, &api.quota)?,
    ))
}

pub async fn kv_get(api: &Api, rep: u32, key: String) -> anyhow::Result<Option<Vec<u8>>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    if check_read(api.doc_id, res.doc).is_err() {
        return Ok(None);
    }
    Ok(replicas::doc_kv_get(ns(res.space), ns(res.doc), &key))
}

pub async fn kv_set(
    api: &Api,
    rep: u32,
    key: String,
    value: Vec<u8>,
) -> anyhow::Result<Result<(), ScriptError>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    if let Err(err) = check_write(api.doc_id, res.doc) {
        return Ok(Err(err.into()));
    }
    Ok(entities::doc_kv_set(ns(res.space), ns(res.doc), key, value)
        .await
        .map_err(Into::into))
}

pub async fn kv_delete(
    api: &Api,
    rep: u32,
    key: String,
) -> anyhow::Result<Result<(), ScriptError>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    if let Err(err) = check_write(api.doc_id, res.doc) {
        return Ok(Err(err.into()));
    }
    Ok(entities::doc_kv_delete(ns(res.space), ns(res.doc), key)
        .await
        .map_err(Into::into))
}

pub async fn kv_keys(api: &Api, rep: u32) -> anyhow::Result<Vec<String>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    if check_read(api.doc_id, res.doc).is_err() {
        return Ok(Vec::new());
    }
    Ok(replicas::doc_kv_keys(ns(res.space), ns(res.doc)))
}

pub async fn kv_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_kv.lock().await.kv_slots.remove(rep);
    Ok(())
}
