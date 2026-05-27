use bevy::prelude::*;
use blake3::Hash;
use loro::TreeID;
use unavi_util::async_commands::AsyncCommands;

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api,
        registry::{
            firewall::validate_firewall,
            transform::AbsoluteNodeId,
        },
    },
};

#[derive(Event, Clone, Debug)]
pub struct OpenPortal {
    pub anchor: AbsoluteNodeId,
    pub source: Hash,
    pub space:  Hash,
}

pub async fn open_portal(api: &Api, prim_rep: u32, space: [u8; 32]) -> anyhow::Result<()> {
    let prim = api
        .wired_scene
        .lock()
        .await
        .prims
        .get(prim_rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid prim rep: {prim_rep}"))?;

    validate_firewall(&api.doc_id, &prim.doc_id, Channel::SceneWrite)?;

    emit_open_portal(api.doc_id, prim.doc_id, prim.id, Hash::from_bytes(space)).await
}

async fn emit_open_portal(
    source: Hash,
    anchor_doc: Hash,
    anchor_node: TreeID,
    space: Hash,
) -> anyhow::Result<()> {
    AsyncCommands::default()
        .trigger(OpenPortal {
            anchor: AbsoluteNodeId {
                doc:  anchor_doc,
                node: anchor_node,
            },
            source,
            space,
        })
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("open-portal send: {e}"))
}
