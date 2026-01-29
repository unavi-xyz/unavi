use bevy::prelude::*;
use bevy_wds::{LocalActor, RemoteActor};
use loro_surgeon::Reconcile;
use time::OffsetDateTime;
use wds::actor::Actor;
use wired_schemas::{SCHEMA_BEACON, SCHEMA_HOME, SCHEMA_SPACE, SCHEMA_STAGE};

use crate::{
    networking::thread::{NetworkCommand, NetworkingThread},
    space::{beacon::Beacon, default_stage::default_stage},
};

pub fn join_home_space(
    local_actor: Query<&LocalActor>,
    remote_actors: Query<&RemoteActor>,
    nt: Res<NetworkingThread>,
    mut did_join: Local<bool>,
) {
    // TODO: Use proper state to track space status, join home if not in any spaces
    if *did_join {
        return;
    }

    let Ok(local_actor) = local_actor.single().map(|x| x.0.clone()) else {
        return;
    };

    let remote_actor = remote_actors.iter().next().map(|x| x.0.clone());

    let command_tx = nt.command_tx.clone();

    unavi_wasm_compat::spawn_thread(async move {
        if let Err(err) = discover_or_home(local_actor, remote_actor, command_tx).await {
            error!(?err, "Failed to join home space");
        }
    });

    *did_join = true;
}

/// Attempt to discover a populated space to join.
/// Else, join the user's home.
/// Temporary measure until proper space traversal exists.
async fn discover_or_home(
    local_actor: Actor,
    remote_actor: Option<Actor>,
    command_tx: tokio::sync::mpsc::Sender<NetworkCommand>,
) -> anyhow::Result<()> {
    if let Some(remote_actor) = &remote_actor {
        match try_fetch_beacons(&local_actor, remote_actor).await {
            Ok(beacons) => {
                if let Some(beacon) = beacons.first() {
                    info!("Found populated space: {}", beacon.space);
                    command_tx.send(NetworkCommand::Join(beacon.space)).await?;
                    return Ok(());
                }
            }
            Err(err) => {
                warn!(?err, "failed to fetch beacons");
            }
        }
    }

    create_and_join_home(local_actor, remote_actor, command_tx).await?;

    Ok(())
}

async fn try_fetch_beacons(
    local_actor: &Actor,
    remote_actor: &Actor,
) -> anyhow::Result<Vec<Beacon>> {
    let mut beacons = Vec::new();

    let found = remote_actor
        .query()
        .schema(SCHEMA_BEACON.hash)
        .send()
        .await?;
    info!("Queried {} beacons", found.len());

    let remote_host = *remote_actor.host();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    for id in found {
        match local_actor
            .read(id)
            .sync_from(remote_host.into())
            .send()
            .await
        {
            Ok(doc) => {
                let Ok(beacon) = Beacon::load(&doc) else {
                    debug!("invalid beacon documnt");
                    continue;
                };

                if now >= beacon.expires {
                    continue;
                }

                beacons.push(beacon);
            }
            Err(err) => {
                warn!(?err, "failed to sync beacon");
            }
        }
    }

    beacons.sort_by(|a, b| a.expires.cmp(&b.expires));

    Ok(beacons)
}

async fn create_and_join_home(
    local_actor: Actor,
    remote_actor: Option<Actor>,
    command_tx: tokio::sync::mpsc::Sender<NetworkCommand>,
) -> anyhow::Result<()> {
    let did = local_actor.identity().did();

    let res = local_actor
        .create_record()
        .public()
        .add_schema("home", &*SCHEMA_HOME, |_| Ok(()))?
        .add_schema("space", &*SCHEMA_SPACE, |doc| {
            let map = doc.get_map("space");
            map.insert("name", format!("{did}'s Home"))?;
            Ok(())
        })?
        .add_schema("hsd", &*SCHEMA_STAGE, |doc| {
            let map = doc.get_map("hsd");
            let stage = default_stage();
            stage.reconcile(&map)?;
            Ok(())
        })?
        .sync_to(remote_actor)
        .send()
        .await?;

    info!("Created home space: {}", res.id);
    command_tx.send(NetworkCommand::Join(res.id)).await?;

    Ok(())
}
