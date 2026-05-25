use std::collections::BTreeSet;

use blake3::Hash;
use iroh::{
    EndpointId,
    PublicKey,
};
use time::OffsetDateTime;
use tracing::warn;
use wired_records::beacon::BeaconRecord;
use wired_schemas::SCHEMA_BEACON;

use crate::gossip::GossipCtx;

pub async fn find_bootstrap_peers(
    ctx: &GossipCtx,
    space: Hash,
) -> anyhow::Result<BTreeSet<PublicKey>> {
    let mut bootstrap = BTreeSet::new();

    // Query beacons from remote actors, with local as fallback.
    let target_actors = if ctx.sync_targets.is_empty() {
        vec![ctx.actor.clone()]
    } else {
        ctx.sync_targets.clone()
    };

    for actor in target_actors {
        let found = actor.query().schema(SCHEMA_BEACON.hash).send().await?;
        let now = OffsetDateTime::now_utc().unix_timestamp();

        for id in found {
            // Read from the local actor.
            let mut builder = ctx.actor.read(id);

            if actor.host() != ctx.actor.host() {
                builder = builder.sync_from(actor.host().clone());
            }

            match builder.send().await {
                Ok(doc) => {
                    let Ok(beacon) = BeaconRecord::load(&doc) else {
                        continue;
                    };
                    if now >= beacon.expires {
                        continue;
                    }
                    if beacon.space.as_bytes() != space.as_bytes() {
                        continue;
                    }
                    let Ok(endpoint) = EndpointId::from_bytes(beacon.endpoint.0.as_bytes()) else {
                        continue;
                    };
                    if endpoint == ctx.endpoint.id() {
                        continue;
                    }
                    bootstrap.insert(endpoint);
                }
                Err(err) => {
                    warn!(?err, "Failed to sync beacon");
                }
            }
        }
    }

    Ok(bootstrap)
}
