use hsd::id::DocId;
use iroh_docs::NamespaceId;
use unavi_policy::quota::{
    Flow,
    Stock,
};
use unavi_space::check::{
    space_of,
    write as check_write,
};
use unavi_util::async_commands::AsyncCommands;

use crate::{
    error::ScriptError,
    portal_host::PortalWatch,
    quota::QuotaGuards,
    runtime::shared::Api,
};

pub async fn open(api: &Api, prim_rep: u32, target_space: Vec<u8>) -> Result<(), ScriptError> {
    crate::quota::acquire(&api.quota, Flow::PortalOpen, 1.0).await?;

    let (doc, tree_id) = {
        let scene = api.wired_scene.lock().await;
        let prim = scene
            .prims
            .get(prim_rep)
            .ok_or_else(|| ScriptError::other(format!("invalid prim rep: {prim_rep}")))?;
        let ids = (prim.doc_id, prim.id.to_string());
        drop(scene);
        ids
    };

    // Opening a portal writes a handshake on behalf of the source prim; the
    // caller must hold scene-write on that prim's document.
    check_write(&api.policy, api.doc_id, doc)?;

    let source_space = space_of(&api.policy, doc)
        .ok_or_else(|| ScriptError::other("prim doc is not in a tracked space"))?;
    let target = <[u8; 32]>::try_from(target_space.as_slice())
        .map_err(|_| ScriptError::other("document id must be 32 bytes"))?;

    let watch_guard = api.quota.charge(Stock::PortalWatches, 1)?;

    AsyncCommands::default()
        .spawn((
            PortalWatch::new(source_space, doc, tree_id, DocId(target)),
            QuotaGuards(vec![watch_guard]),
        ))
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    Ok(())
}

pub async fn travel(api: &Api, target_space: Vec<u8>) -> Result<(), ScriptError> {
    crate::quota::acquire(&api.quota, Flow::PortalOpen, 1.0).await?;

    let target = <[u8; 32]>::try_from(target_space.as_slice())
        .map_err(|_| ScriptError::other("document id must be 32 bytes"))?;
    let hash = NamespaceId::from(&target);

    AsyncCommands::default()
        .push(move |world: &mut bevy::prelude::World| {
            unavi_space::travel::request_travel(world, hash);
        })
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    Ok(())
}
