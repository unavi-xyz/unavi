use std::cell::RefCell;

use bevy::prelude::*;
use bevy_wds::{LocalActor, SyncTargets};
use wds::actor::Actor;
use wired_schemas::{SCHEMA_HOME, SCHEMA_HSD, SCHEMA_SPACE};

#[derive(Default)]
pub struct JoinState {
    ready: bool,
    joined: bool,
}

use crate::{
    networking::thread::{NetworkCommand, NetworkingThread},
    space::default_space::default_space,
};

pub fn join_home_space(
    local_actor: Query<(&LocalActor, &SyncTargets)>,
    nt: Res<NetworkingThread>,
    mut state: Local<JoinState>,
) {
    if state.joined {
        return;
    }

    let Ok((local_actor, sync_targets)) = local_actor.single() else {
        return;
    };

    // Wait one frame after LocalActor spawns so recv_network_event
    // can drain any pending remote actors into SyncTargets.
    if !state.ready {
        state.ready = true;
        return;
    }

    let local_actor = local_actor.0.clone();
    let remote_actor = sync_targets.0.first().cloned();

    let command_tx = nt.command_tx.clone();

    unavi_wasm_compat::spawn_thread(async move {
        if let Err(err) = create_and_join_home(local_actor, remote_actor, command_tx).await {
            error!(?err, "Failed to join home space");
        }
    });

    state.joined = true;
}

async fn create_and_join_home(
    local_actor: Actor,
    remote_actor: Option<Actor>,
    command_tx: tokio::sync::mpsc::Sender<NetworkCommand>,
) -> anyhow::Result<()> {
    let did = local_actor.identity().did();

    let blobs = RefCell::new(None);

    let res = local_actor
        .create_record()
        .public()
        .add_schema("home", &*SCHEMA_HOME, |_| Ok(()))?
        .add_schema("space", &*SCHEMA_SPACE, |doc| {
            let map = doc.get_map("space");
            map.insert("name", format!("{did}'s Home"))?;
            Ok(())
        })?
        .add_schema("hsd", &*SCHEMA_HSD, |doc| {
            let hsd = doc.get_map("hsd");
            *blobs.borrow_mut() = Some(default_space(&hsd)?);
            Ok(())
        })?
        .sync_to(remote_actor.clone())
        .send()
        .await?;

    let blobs = blobs.into_inner().unwrap_or_default();

    for bytes in blobs.0 {
        if let Some(remote_actor) = &remote_actor
            && let Err(err) = remote_actor.upload_blob(bytes.clone()).await
        {
            warn!(?err, "failed to upload blob dep to remote");
        }

        local_actor.upload_blob(bytes).await?;
    }

    info!(id = %res.id, "Created home space");
    command_tx.send(NetworkCommand::Join(res.id)).await?;

    Ok(())
}
