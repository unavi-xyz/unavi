use std::collections::HashSet;

use bevy::{ecs::world::CommandQueue, tasks::futures_lite::StreamExt};
use blake3::Hash;
use iroh_gossip::TopicId;
use log::{debug, warn};
use time::OffsetDateTime;
use wds::record::schema::SCHEMA_BEACON;

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    networking::thread::NetworkThreadState,
    space::{Space, beacon::Beacon},
};

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

    // Join gossip topic.
    let topic_id = TopicId::from_bytes(*id.as_bytes());
    let topic = state
        .gossip
        .subscribe(topic_id, bootstrap.into_iter().collect())
        .await?;
    let (_tx, mut rx) = topic.split();

    // Create space in ECS.
    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([(Space(id))]));
    ASYNC_COMMAND_QUEUE.0.send_async(commands).await?;

    // Recieve gossip.
    while let Some(e) = rx.next().await {
        let _e = e?;
    }

    Ok(())
}
