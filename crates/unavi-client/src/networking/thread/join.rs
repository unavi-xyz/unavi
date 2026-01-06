use std::{collections::HashSet, time::Duration};

use bevy::tasks::futures_lite::StreamExt;
use blake3::Hash;
use iroh_gossip::TopicId;
use log::{debug, warn};
use time::OffsetDateTime;
use wds::record::schema::SCHEMA_BEACON;

use crate::{networking::thread::NetworkThreadState, space::beacon::Beacon};

const BEACON_TTL: Duration = Duration::from_secs(30);

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

    // Create our own beacon.
    // TODO: Loop while we are in the space
    let _res = state
        .local_actor
        .create_record()
        .ttl(BEACON_TTL)
        .add_schema(&*SCHEMA_BEACON, |doc| {
            let beacon = Beacon {
                did: state.local_actor.identity().did().clone(),
                expires: (OffsetDateTime::now_utc() + BEACON_TTL).unix_timestamp(),
                endpoint: state.endpoint_id,
                space: id,
            };
            beacon.save(doc)?;
            Ok(())
        })?
        .sync_to(state.remote_actor)
        .send()
        .await?;

    while let Some(e) = rx.next().await {
        let _e = e?;
    }

    Ok(())
}
