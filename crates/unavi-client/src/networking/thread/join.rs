use std::time::Duration;

use blake3::Hash;
use time::OffsetDateTime;
use wds::record::schema::SCHEMA_BEACON;

use crate::networking::thread::NetworkThreadState;

const BEACON_TTL: Duration = Duration::from_secs(30);

pub async fn handle_join(state: NetworkThreadState, id: Hash) -> anyhow::Result<()> {
    // Query WDS for beacons.

    // Connect to beacons.

    // Create our own beacon.
    // TODO: Loop while we are in the space
    let _res = state
        .local_actor
        .create_record()
        .ttl(BEACON_TTL)
        .add_schema(&*SCHEMA_BEACON, |doc| {
            let map = doc.get_map("beacon");
            map.insert("did", state.local_actor.did().to_string())?;
            // map.insert("endpoint", state.endpoint_id)?;
            map.insert(
                "expires",
                (OffsetDateTime::now_utc() + BEACON_TTL).unix_timestamp(),
            )?;
            map.insert("space", id.as_bytes().to_vec())?;
            Ok(())
        })?
        .sync_to(state.remote_actor)
        .send()
        .await?;

    Ok(())
}
