use bevy::prelude::*;
use blake3::Hash;
use unavi_space::Space;
use unavi_util::async_commands::AsyncCommands;

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api,
        registry::firewall::validate_firewall,
    },
};

pub async fn open_portal(api: &Api, prim_rep: u32, dest: [u8; 32]) -> anyhow::Result<()> {
    let prim = api
        .wired_scene
        .lock()
        .await
        .prims
        .get(prim_rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid prim rep: {prim_rep}"))?;

    validate_firewall(&api.doc_id, &prim.doc_id, Channel::SceneWrite)?;

    let dest = Hash::from_bytes(dest);

    AsyncCommands::default()
        .push(move |world: &mut World| {
            for space in world.query::<&Space>().query(world) {
                // Only allow a single instance of each space.
                // We run as an exclusive system to ensure singularity.
                if space.0 == dest {
                    return;
                }
            }
            world.spawn(Space(dest));
        })
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("open-portal send: {e}"))
}
