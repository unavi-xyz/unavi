use bevy::prelude::*;
use iroh_docs::NamespaceId;
use unavi_space::{
    membership::doc_space,
    state::{
        entities,
        replicas::{
            self,
            KvError,
        },
    },
};

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api,
        registry::firewall::validate_firewall,
        slot_map::SlotMap,
    },
};

#[derive(Clone, Copy)]
pub struct KvRes {
    pub space: NamespaceId,
    pub doc:   NamespaceId,
}

#[derive(Default)]
pub struct WiredKvApi {
    kv_slots: SlotMap<KvRes>,
}

pub async fn self_kv(api: &Api) -> anyhow::Result<u32> {
    let Some(space) = doc_space(api.doc_id) else {
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
    let Some(space) = doc_space(api.doc_id) else {
        return Ok(None);
    };
    let Ok(bytes) = <[u8; 32]>::try_from(doc_id.as_slice()) else {
        return Ok(None);
    };
    let doc = NamespaceId::from(&bytes);

    if !replicas::has_doc(space, doc) {
        return Ok(None);
    }

    if validate_firewall(&api.doc_id, &doc, Channel::KvRead).is_err() {
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
    if validate_firewall(&api.doc_id, &res.doc, Channel::KvRead).is_err() {
        return Ok(None);
    }
    Ok(replicas::doc_kv_get(res.space, res.doc, &key))
}

pub async fn kv_set(
    api: &Api,
    rep: u32,
    key: String,
    value: Vec<u8>,
) -> anyhow::Result<Result<(), KvError>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    if validate_firewall(&api.doc_id, &res.doc, Channel::KvWrite).is_err() {
        return Ok(Err(KvError::Other));
    }
    Ok(entities::doc_kv_set(res.space, res.doc, key, value).await)
}

pub async fn kv_delete(api: &Api, rep: u32, key: String) -> anyhow::Result<Result<(), KvError>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    if validate_firewall(&api.doc_id, &res.doc, Channel::KvWrite).is_err() {
        return Ok(Err(KvError::Other));
    }
    Ok(entities::doc_kv_delete(res.space, res.doc, key).await)
}

pub async fn kv_keys(api: &Api, rep: u32) -> anyhow::Result<Vec<String>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    if validate_firewall(&api.doc_id, &res.doc, Channel::KvRead).is_err() {
        return Ok(Vec::new());
    }
    Ok(replicas::doc_kv_keys(res.space, res.doc))
}

pub async fn kv_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_kv.lock().await.kv_slots.remove(rep);
    Ok(())
}
