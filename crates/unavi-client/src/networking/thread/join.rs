use std::{collections::HashSet, time::Duration};

use blake3::Hash;
use log::{debug, info, warn};
use time::OffsetDateTime;
use wds::record::schema::SCHEMA_BEACON;

use crate::{networking::thread::NetworkThreadState, space::beacon::Beacon};

const BEACON_TTL: Duration = Duration::from_secs(30);

pub async fn handle_join(state: NetworkThreadState, id: Hash) -> anyhow::Result<()> {
    // Query WDS for beacons.
    let mut players = HashSet::new();

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

                    if now >= beacon.expires {
                        continue;
                    }

                    players.insert(beacon.endpoint);
                }
                Err(e) => {
                    warn!("failed to sync beacon: {e:?}");
                }
            }
        }
    }

    // Connect to players.
    for endpoint in players {
        info!("Found player: {endpoint}");
        // TODO
    }

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

    Ok(())
}
