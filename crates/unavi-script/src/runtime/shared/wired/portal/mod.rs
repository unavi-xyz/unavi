use hsd::{
    attributes::portal::{
        LinkId,
        PortalDestination,
    },
    id::{
        DocId,
        PrimId,
    },
};
use iroh_docs::NamespaceId;
use unavi_policy::quota::Flow;

use crate::{
    error::ScriptError,
    runtime::shared::{
        Api,
        wired::scene::prim::set_session_destination,
    },
};

#[cfg(test)] mod tests;

pub async fn open(api: &Api, prim_rep: u32, target_space: Vec<u8>) -> Result<(), ScriptError> {
    crate::quota::acquire(&api.quota, Flow::PortalOpen, 1).await?;

    let target = space_id(&target_space)?;
    let (doc, prim) = {
        let scene = api.wired_scene.lock().await;
        scene
            .prims
            .get(prim_rep)
            .map(|prim| (prim.doc_id, prim.id))
            .ok_or_else(|| ScriptError::other(format!("invalid prim rep: {prim_rep}")))?
    };

    let destination = PortalDestination {
        space: target,
        link:  Some(derive_link(doc, prim, target)),
    };
    set_session_destination(api, prim_rep, destination)
        .await
        .map_err(ScriptError::from)?
}

pub async fn pair(
    api: &Api,
    prim_rep: u32,
    source_space: Vec<u8>,
    link: Vec<u8>,
) -> Result<(), ScriptError> {
    crate::quota::acquire(&api.quota, Flow::PortalOpen, 1).await?;

    let link = <[u8; 16]>::try_from(link.as_slice())
        .map(LinkId)
        .map_err(|_| ScriptError::other("link id must be 16 bytes"))?;
    let destination = PortalDestination {
        space: space_id(&source_space)?,
        link:  Some(link),
    };
    set_session_destination(api, prim_rep, destination)
        .await
        .map_err(ScriptError::from)?
}

pub async fn travel(api: &Api, target_space: Vec<u8>) -> Result<(), ScriptError> {
    crate::quota::acquire(&api.quota, Flow::PortalOpen, 1).await?;

    let target = space_id(&target_space)?;
    let hash = NamespaceId::from(&target);

    api.async_world
        .commands()
        .push(move |world: &mut bevy::prelude::World| {
            unavi_space::travel::request_travel(world, hash.into());
        })
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    Ok(())
}

fn space_id(bytes: &[u8]) -> Result<[u8; 32], ScriptError> {
    <[u8; 32]>::try_from(bytes).map_err(|_| ScriptError::other("document id must be 32 bytes"))
}

/// The same on every peer that opens `prim` into `target`, since each runs the
/// opening script and a per-call id would leave the session cell and the far
/// half disagreeing.
fn derive_link(doc: DocId, prim: PrimId, target: [u8; 32]) -> LinkId {
    let mut hasher = blake3::Hasher::new_derive_key("unavi portal link");
    hasher.update(&doc.0);
    hasher.update(&prim.0);
    hasher.update(&target);
    let mut link = [0; 16];
    link.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    LinkId(link)
}
