use std::time::Duration;

use bevy::prelude::*;
use bevy_iroh::endpoint::IrohEndpoint;
use bevy_wds::{
    LocalActor,
    SyncTargets,
};
use time::OffsetDateTime;
use unavi_util::async_task::spawn_async_task;
use wds::format::Beacon;

use crate::Space;

const BEACON_TTL: Duration = Duration::from_mins(2);

pub fn publish_beacons(
    time: Res<Time>,
    spaces: Query<&Space>,
    actors: Query<(&LocalActor, &SyncTargets)>,
    endpoint: Query<&IrohEndpoint>,
    mut last: Local<Duration>,
) {
    if spaces.is_empty() {
        return;
    }

    let now = time.elapsed();
    if !last.is_zero() && now.saturating_sub(*last) < BEACON_TTL {
        return;
    }
    *last = now;

    let Ok((actor, sync_targets)) = actors.single() else {
        return;
    };
    let Ok(endpoint) = endpoint.single() else {
        return;
    };

    let did = actor.0.identity().did().clone();
    let endpoint_id = *endpoint.0.id().as_bytes();

    // Announce to the home servers we sync with; fall back to the local host.
    let targets = if sync_targets.0.is_empty() {
        vec![actor.0.clone()]
    } else {
        sync_targets.0.clone()
    };

    for space in spaces {
        let beacon = Beacon {
            did:      did.clone(),
            endpoint: endpoint_id,
            space:    space.0,
            expires:  (OffsetDateTime::now_utc() + BEACON_TTL).unix_timestamp(),
        };

        for target in &targets {
            let target = target.clone();
            let beacon = beacon.clone();
            spawn_async_task(async move {
                if let Err(err) = target.announce(beacon).await {
                    warn!(?err, "Failed to announce beacon");
                }
            });
        }
    }
}
