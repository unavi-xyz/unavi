use std::time::Duration;

use blake3::Hash;
use log::info;
use time::OffsetDateTime;
use wds::record::schema::SCHEMA_BEACON;

use crate::{networking::thread::NetworkThreadState, space::beacon::Beacon};

pub async fn publish_beacon(
    state: NetworkThreadState,
    id: Hash,
    ttl: Duration,
) -> anyhow::Result<()> {
    let res = state
        .local_actor
        .create_record()
        .ttl(ttl)
        .add_schema(&*SCHEMA_BEACON, |doc| {
            let beacon = Beacon {
                did: state.local_actor.identity().did().clone(),
                expires: (OffsetDateTime::now_utc() + ttl).unix_timestamp(),
                endpoint: state.endpoint_id,
                space: id,
            };
            beacon.save(doc)?;
            Ok(())
        })?
        .public()
        .sync_to(state.remote_actor)
        .send()
        .await?;

    info!("published beacon: space={} id={}", id, res.id);

    Ok(())
}
