use std::collections::HashSet;

use bevy::{
    ecs::world::CommandQueue,
    log::{debug, info, warn},
    tasks::futures_lite::StreamExt,
};
use blake3::Hash;
use iroh::EndpointId;
use iroh_gossip::{TopicId, api::Event};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use wds::record::schema::SCHEMA_BEACON;

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    networking::thread::NetworkThreadState,
    space::{Space, beacon::Beacon},
};

#[derive(Serialize, Deserialize)]
struct JoinBroadcast {
    endpoint: EndpointId,
}

pub async fn handle_join(state: NetworkThreadState, id: Hash) -> anyhow::Result<()> {
    // Query beacons to find players.
    let mut bootstrap = HashSet::new();

    if let Some(actor) = &state.remote_actor {
        let found = actor.query().schema(SCHEMA_BEACON.hash).send().await?;

        let remote_host = *actor.host();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        for id in found {
            match state
                .local_actor
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

                    // Ignore old beacons.
                    if now >= beacon.expires {
                        continue;
                    }

                    // Ignore our own beacon.
                    if beacon.endpoint == state.endpoint_id {
                        continue;
                    }

                    bootstrap.insert(beacon.endpoint);
                }
                Err(e) => {
                    warn!("failed to sync beacon: {e:?}");
                }
            }
        }
    }

    info!("Bootstrapping space: peers={bootstrap:?}");

    // Join gossip topic.
    let topic_id = TopicId::from_bytes(*id.as_bytes());
    let topic = state
        .gossip
        .subscribe(topic_id, bootstrap.into_iter().collect())
        .await?;
    let (tx, mut rx) = topic.split();

    // Create space in ECS.
    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([(Space(id))]));
    ASYNC_COMMAND_QUEUE.0.send_async(commands).await?;

    // Broadcast join.
    let join = postcard::to_stdvec(&JoinBroadcast {
        endpoint: state.endpoint_id,
    })?;
    tx.broadcast(join.into()).await?;

    // Recieve gossip.
    while let Some(e) = rx.next().await {
        match e? {
            Event::NeighborUp(n) => {
                info!(?id, "+neighbor: {n}");
            }
            Event::NeighborDown(n) => {
                info!(?id, "-neighbor: {n}");
            }
            Event::Lagged => {}
            Event::Received(m) => match postcard::from_bytes::<JoinBroadcast>(&m.content) {
                Ok(m) => {
                    info!(?id, "Player joined: {}", m.endpoint);
                    // TODO: connect to player
                }
                Err(e) => {
                    warn!(?id, "Got invalid gossip message: {e:?}");
                }
            },
        }
    }

    Ok(())
}
