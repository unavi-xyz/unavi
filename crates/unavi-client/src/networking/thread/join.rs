use blake3::Hash;
use wds::record::schema::SCHEMA_BEACON;

use crate::networking::thread::NetworkThreadState;

pub async fn handle_join(state: NetworkThreadState, _id: Hash) -> anyhow::Result<()> {
    // Query WDS for beacons.

    // Connect to beacons.

    // Create our own beacon.
    let (_beacon_id, _doc) = state
        .actor
        .create_record(Some(vec![*SCHEMA_BEACON]))
        .await?;

    todo!()
}
