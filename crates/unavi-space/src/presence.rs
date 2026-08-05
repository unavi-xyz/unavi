use std::time::Duration;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_iroh::endpoint::IrohEndpoint;
use bevy_wds::{
    LocalActor,
    registry_clients,
};
use iroh_docs::NamespaceId;
use time::OffsetDateTime;
use unavi_util::async_task::spawn_async_task;
use wired_registry::entry::Presence;

use crate::Space;

const PRESENCE_TTL: Duration = Duration::from_mins(2);

/// Republishing on the TTL itself lets every entry lapse before its
/// replacement lands, and a registry filters expired presence — so a peer that
/// never left reads as absent for the width of that gap, and whoever queries
/// inside it finds nobody to bootstrap against.
const PRESENCE_REFRESH: Duration = PRESENCE_TTL.checked_div(3).expect("nonzero divisor");

/// Heartbeats this peer's occupancy of each active space to every registry it
/// follows, so others can find peers to bootstrap gossip against.
/// Tracked per space, not per process: entering a space is exactly when the
/// peers already in it need to hear about us, and a shared timer would hold the
/// announcement back until whenever the last one happened to fall due.
pub fn publish_presence(
    time: Res<Time>,
    spaces: Query<&Space>,
    actors: Query<&LocalActor>,
    endpoint: Query<&IrohEndpoint>,
    mut last: Local<HashMap<NamespaceId, Duration>>,
) {
    if spaces.is_empty() {
        return;
    }

    let now = time.elapsed();
    let due = spaces
        .iter()
        .filter(|space| {
            last.get(&space.0)
                .is_none_or(|last| now.saturating_sub(*last) >= PRESENCE_REFRESH)
        })
        .collect::<Vec<_>>();

    if due.is_empty() {
        return;
    }

    let Ok(actor) = actors.single() else {
        return;
    };
    let Ok(endpoint) = endpoint.single() else {
        return;
    };

    // Registries load asynchronously at startup, so the first spaces usually
    // exist before there is anywhere to announce to. The interval is stamped
    // only once an announcement actually goes out; stamping it earlier would
    // silently defer the first real publish by a full TTL.
    let registries = registry_clients();
    if registries.is_empty() {
        return;
    }

    let did = actor.0.identity().did().clone();
    let endpoint_id = *endpoint.0.id().as_bytes();

    for space in due {
        last.insert(space.0, now);

        let presence = Presence {
            did:      did.clone(),
            endpoint: endpoint_id,
            ns:       space.0,
            expires:  (OffsetDateTime::now_utc() + PRESENCE_TTL).unix_timestamp(),
        };

        for registry in &registries {
            let registry = registry.clone();
            let presence = presence.clone();
            spawn_async_task(async move {
                if let Err(err) = registry.announce(&presence).await {
                    warn!(?err, "Failed to announce presence");
                }
            });
        }
    }
}
