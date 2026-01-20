use bevy::prelude::*;
use time::OffsetDateTime;
use wired_schemas::{SCHEMA_BEACON, SCHEMA_HOME, SCHEMA_SPACE};

use crate::{
    networking::{
        WdsActors,
        thread::{NetworkCommand, NetworkingThread},
    },
    space::beacon::Beacon,
};

pub fn join_home_space(
    actors: Query<&WdsActors>,
    nt: Res<NetworkingThread>,
    mut did_join: Local<bool>,
) {
    // TODO: Use proper state to track space status, join home if not in any spaces
    if *did_join {
        return;
    }

    let Ok(actors) = actors.single().cloned() else {
        warn!("WDS actors not found");
        return;
    };

    let command_tx = nt.command_tx.clone();

    unavi_wasm_compat::spawn_thread(async move {
        if let Err(e) = discover_or_home(actors, command_tx).await {
            error!("Failed to join home space: {e:?}");
        }
    });

    *did_join = true;
}

/// Attempt to discover a populated space to join.
/// Else, join the user's home.
/// Temporary measure until proper space traversal exists.
async fn discover_or_home(
    actors: WdsActors,
    command_tx: tokio::sync::mpsc::Sender<NetworkCommand>,
) -> anyhow::Result<()> {
    // Query for beacons.
    let mut beacons = Vec::new();

    if let Some(remote_actor) = &actors.remote {
        let found = remote_actor
            .query()
            .schema(SCHEMA_BEACON.hash)
            .send()
            .await?;
        info!("Queried {} beacons", found.len());

        let remote_host = *remote_actor.host();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        for id in found {
            match actors
                .local
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
                Err(e) => {
                    warn!("failed to sync beacon: {e:?}");
                }
            }
        }
    }

    beacons.sort_by(|a, b| a.expires.cmp(&b.expires));

    if let Some(beacon) = beacons.first() {
        // Join most recent beacon.
        info!("Found populated space: {}", beacon.space);
        command_tx.send(NetworkCommand::Join(beacon.space)).await?;
    } else {
        // Join home.
        join_home_space_inner(actors, command_tx).await?;
    }

    Ok(())
}

async fn join_home_space_inner(
    actors: WdsActors,
    command_tx: tokio::sync::mpsc::Sender<NetworkCommand>,
) -> anyhow::Result<()> {
    let did = actors.local.identity().did();

    let res = actors
        .local
        .create_record()
        .add_schema("home", &*SCHEMA_HOME, |_| Ok(()))?
        .add_schema("space", &*SCHEMA_SPACE, |doc| {
            let map = doc.get_map("space");
            map.insert("name", format!("{did}'s Home"))?;
            Ok(())
        })?
        .sync_to(actors.remote)
        .send()
        .await?;

    info!("Created home space: {}", res.id);
    command_tx.send(NetworkCommand::Join(res.id)).await?;

    Ok(())
}
