use std::time::Duration;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_iroh::endpoint::IrohEndpoint;
use iroh_docs::NamespaceId;
use time::OffsetDateTime;
use unavi_policy::space::Space;
use unavi_registry::{
    entry::Presence,
    follow::registry_clients,
};
use unavi_util::async_task::spawn_async_task;

const PRESENCE_TTL: Duration = Duration::from_mins(2);

/// Kept under `PRESENCE_TTL` so an entry never lapses into absent presence
/// before its replacement lands.
const PRESENCE_REFRESH: Duration = PRESENCE_TTL.checked_div(3).expect("nonzero divisor");

/// Heartbeats occupancy of each active space to every followed registry, so
/// others can find peers to bootstrap gossip against. Tracked per space, not
/// per process: a shared timer delays a new space's announcement until the last
/// space's timer falls due.
pub fn publish_presence(
    time: Res<Time>,
    spaces: Query<&Space>,
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

    let Ok(endpoint) = endpoint.single() else {
        return;
    };

    // Registries load asynchronously at startup, so spaces usually precede
    // them. The interval is stamped only when an announcement actually goes
    // out; stamping earlier would defer the first real publish by a full TTL.
    let registries = registry_clients();
    if registries.is_empty() {
        return;
    }

    let Some(did) = unavi_identity::identity::local_did() else {
        return;
    };
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
