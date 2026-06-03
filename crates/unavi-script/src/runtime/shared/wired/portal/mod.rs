use anyhow::{
    Result,
    anyhow,
    bail,
};
use unavi_portal_protocol::{
    LinkState,
    link_kv_key,
};
use unavi_space::{
    membership::doc_space,
    state::doc::{
        KvError,
        doc_kv_get,
        doc_kv_set,
    },
};

use crate::runtime::shared::Api;

pub async fn open(api: &Api, prim_rep: u32, target_space: Vec<u8>) -> Result<()> {
    let (doc, tree_id) = {
        let scene = api.wired_scene.lock().await;
        let prim = scene
            .prims
            .get(prim_rep)
            .ok_or_else(|| anyhow!("invalid prim rep: {prim_rep}"))?;
        (prim.doc_id, prim.id.to_string())
    };
    let space = doc_space(doc).ok_or_else(|| anyhow!("prim doc is not in a tracked space"))?;
    let target = <[u8; 32]>::try_from(target_space.as_slice())
        .map_err(|_| anyhow!("document id must be 32 bytes"))?;
    let key = link_kv_key(&tree_id);

    let existing =
        doc_kv_get(space, doc, &key).and_then(|b| postcard::from_bytes::<LinkState>(&b).ok());
    if let Some(state) = &existing
        && state.target_space == target
    {
        return Ok(());
    }
    let next = LinkState {
        target_space:  target,
        receptor_doc:  None,
        receptor_prim: None,
    };
    let bytes = postcard::to_allocvec(&next)?;
    match doc_kv_set(space, doc, &key, &bytes) {
        Ok(()) => Ok(()),
        Err(KvError::KeyTooLong) => bail!("portal: kv key too long"),
        Err(KvError::QuotaExceeded) => bail!("portal: kv quota exceeded"),
        Err(KvError::Other) => bail!("portal: kv error"),
    }
}
