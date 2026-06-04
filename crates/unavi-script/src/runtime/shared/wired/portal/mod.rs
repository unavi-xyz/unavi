use anyhow::{
    Result,
    anyhow,
};
use blake3::Hash;
use unavi_space::membership::doc_space;
use unavi_util::async_commands::AsyncCommands;

use crate::{
    portal_host::PortalWatch,
    runtime::shared::Api,
};

pub async fn open(api: &Api, prim_rep: u32, target_space: Vec<u8>) -> Result<()> {
    let (doc, tree_id) = {
        let scene = api.wired_scene.lock().await;
        let prim = scene
            .prims
            .get(prim_rep)
            .ok_or_else(|| anyhow!("invalid prim rep: {prim_rep}"))?;
        let ids = (prim.doc_id, prim.id.to_string());
        drop(scene);
        ids
    };
    let source_space =
        doc_space(doc).ok_or_else(|| anyhow!("prim doc is not in a tracked space"))?;
    let target = <[u8; 32]>::try_from(target_space.as_slice())
        .map_err(|_| anyhow!("document id must be 32 bytes"))?;

    AsyncCommands::default()
        .spawn(PortalWatch::new(
            source_space,
            doc,
            tree_id,
            Hash::from(target),
        ))
        .send()
        .await?;
    Ok(())
}
