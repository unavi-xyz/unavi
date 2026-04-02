use std::time::Duration;

use bevy::log::info;
use blake3::Hash;
use time::OffsetDateTime;
use wired_schemas::schemas::SCHEMA_BEACON;

use wired_records::{BeaconRecord, HydratedDid, HydratedEndpoint, HydratedHash};

use crate::networking::thread::NetworkThreadState;

pub async fn publish_beacon(
    state: NetworkThreadState,
    id: Hash,
    ttl: Duration,
) -> anyhow::Result<()> {
    let res = state
        .local_actor
        .create_record()
        .public()
        .ttl(ttl)
        .add_schema("beacon", &*SCHEMA_BEACON, |doc| {
            let beacon = BeaconRecord {
                did: HydratedDid(state.local_actor.identity().did().to_string()),
                endpoint: HydratedEndpoint(*state.endpoint.id().as_bytes()),
                expires: (OffsetDateTime::now_utc() + ttl).unix_timestamp(),
                space: HydratedHash(*id.as_bytes()),
            };
            beacon.save(doc)?;
            Ok(())
        })?
        .sync_to(state.remote_actor)
        .send()
        .await?;

    info!(space = %res.id, beacon = %id, "published beacon");

    Ok(())
}
