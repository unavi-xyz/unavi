use blake3::Hash;
use unavi_space::{
    membership::doc_space,
    state::doc::{
        self,
        KvError,
    },
};

use crate::runtime::shared::{
    Api,
    slot_map::SlotMap,
};

#[derive(Clone, Copy)]
pub struct KvRes {
    pub space: Hash,
    pub doc:   Hash,
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
    Ok(slots.kv_slots.insert(KvRes {
        space,
        doc: api.doc_id,
    }))
}

pub async fn get_kv(api: &Api, doc_id: Vec<u8>) -> anyhow::Result<Option<u32>> {
    let Some(space) = doc_space(api.doc_id) else {
        return Ok(None);
    };
    let Ok(bytes) = <[u8; 32]>::try_from(doc_id.as_slice()) else {
        return Ok(None);
    };
    let doc = Hash::from(bytes);

    if !doc::has_doc(space, doc) {
        return Ok(None);
    }

    let mut slots = api.wired_kv.lock().await;
    Ok(Some(slots.kv_slots.insert(KvRes { space, doc })))
}

pub async fn kv_get(api: &Api, rep: u32, key: String) -> anyhow::Result<Option<Vec<u8>>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    Ok(doc::doc_kv_get(res.space, res.doc, &key))
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
    Ok(doc::doc_kv_set(res.space, res.doc, &key, &value))
}

pub async fn kv_delete(api: &Api, rep: u32, key: String) -> anyhow::Result<()> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    doc::doc_kv_delete(res.space, res.doc, &key);
    Ok(())
}

pub async fn kv_keys(api: &Api, rep: u32) -> anyhow::Result<Vec<String>> {
    let slots = api.wired_kv.lock().await;
    let Some(res) = slots.kv_slots.get(rep).copied() else {
        anyhow::bail!("invalid kv resource");
    };
    drop(slots);
    Ok(doc::doc_kv_keys(res.space, res.doc))
}

pub async fn kv_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_kv.lock().await.kv_slots.remove(rep);
    Ok(())
}
