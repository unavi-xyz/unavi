//! Registry occupancy: announcing the space this node stands in, and asking
//! who else is in a space to bootstrap its gossip topic.

use std::{
    collections::BTreeSet,
    sync::Arc,
    time::Duration,
};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_async::task;
use bevy_hsd::document::Hsd;
use bevy_iroh::{
    endpoint::IrohEndpoint,
    store::DataStore,
};
use iroh::PublicKey;
use time::OffsetDateTime;
use unavi_identity::authorship::Authorship;
use unavi_registry::claim::{
    Kind,
    Presence,
    Submission,
};

use crate::{
    discovery::gossip::GossipCtx,
    grid::ActiveSpace,
    identity::LocalIdentity,
    membership::{
        Space,
        SpaceId,
    },
};

const PRESENCE_TTL: Duration = Duration::from_mins(2);

/// Kept under `PRESENCE_TTL` so an entry never lapses before its replacement
/// lands.
const PRESENCE_REFRESH: Duration = PRESENCE_TTL.checked_div(3).expect("nonzero divisor");

/// How long a published space's registry listing lasts.
const LISTING_TTL: Duration = Duration::from_hours(24);

/// How long to wait before re-asserting a listing, well under its TTL.
const LISTING_REFRESH: Duration = Duration::from_hours(12);

/// Publishes every loaded space whose meta declares it public and whose
/// write key this node holds.
///
/// Publishing is how a space becomes discoverable: the registry only lists a
/// namespace named by a live `Kind::Space` submission, and its `active` view
/// is presence intersected with those listings. The meta flag is a
/// declaration; the signed listing is the fact.
pub fn publish_spaces(
    time: Res<Time>,
    spaces: Query<(&Space, &Hsd)>,
    store: Option<Res<DataStore>>,
    identity: Option<Res<LocalIdentity>>,
    mut last: Local<HashMap<SpaceId, Duration>>,
) {
    let (Some(store), Some(local)) = (store, identity) else {
        return;
    };
    // Registries load asynchronously; the interval is stamped only once a
    // publish actually goes out.
    let registries = local.followed.clients();
    if registries.is_empty() {
        return;
    }
    let now = time.elapsed();

    for (space, hsd) in &spaces {
        let Some(meta) = hsd.meta() else {
            continue;
        };
        if !meta.public {
            continue;
        }
        let id = space.id();
        if last
            .get(&id)
            .is_some_and(|when| now.saturating_sub(*when) < LISTING_REFRESH)
        {
            continue;
        }
        last.insert(id, now);

        let ns = space.namespace();
        let submission = Submission {
            did: local.identity.did().clone(),
            ns,
            kind: Kind::Space,
            title: meta.name.clone().unwrap_or_else(|| "space".into()),
            description: meta.description.clone(),
            tags: Vec::new(),
            expires: (OffsetDateTime::now_utc() + LISTING_TTL).unix_timestamp(),
        };
        let store = store.0.clone();
        let identity = Arc::clone(&local.identity);

        for registry in &registries {
            let registry = registry.clone();
            let submission = submission.clone();
            let store = store.clone();
            let identity = Arc::clone(&identity);
            task::spawn(async move {
                // Only a write-key holder may submit, so a public flag on
                // someone else's space does nothing here.
                let Ok(Some(key)) = store.write_key(ns).await else {
                    return;
                };
                let Ok(authorship) = Authorship::sign(&key, &identity) else {
                    return;
                };
                if let Err(err) = registry.submit(&submission, authorship).await {
                    warn!(?err, %ns, "Failed to publish space");
                }
            });
        }
    }
}

/// Heartbeats occupancy of the active space to every followed registry.
///
/// Others find peers to bootstrap gossip against this way. Only the active
/// space: one held through a portal but not entered never lists this node's
/// DID.
pub fn announce_presence(
    time: Res<Time>,
    active: Res<ActiveSpace>,
    spaces: Query<&Space>,
    endpoint: Query<&IrohEndpoint>,
    identity: Option<Res<LocalIdentity>>,
    mut last: Local<Option<(SpaceId, Duration)>>,
) {
    let Some(space) = active.0.and_then(|e| spaces.get(e).ok()).map(Space::id) else {
        return;
    };
    let now = time.elapsed();
    if last.is_some_and(|(at, when)| at == space && now.saturating_sub(when) < PRESENCE_REFRESH) {
        return;
    }

    let Ok(endpoint) = endpoint.single() else {
        return;
    };
    let Some(local) = identity else {
        return;
    };

    // Registries load asynchronously at startup, so spaces usually precede
    // them. The interval is stamped only when an announcement actually goes
    // out; stamping earlier would defer the first real publish by a full TTL.
    let registries = local.followed.clients();
    if registries.is_empty() {
        return;
    }
    *last = Some((space, now));

    let presence = Presence {
        did:      local.identity.did().clone(),
        endpoint: endpoint.0.id(),
        ns:       space.namespace(),
        expires:  (OffsetDateTime::now_utc() + PRESENCE_TTL).unix_timestamp(),
    };

    for registry in registries {
        let presence = presence.clone();
        task::spawn(async move {
            if let Err(err) = registry.announce(&presence).await {
                warn!(?err, "Failed to announce presence");
            }
        });
    }
}

/// Peers each followed registry lists in `space`, to bootstrap gossip
/// against. The registry client has verified each entry and its expiry.
pub(super) async fn find_bootstrap_peers(ctx: &GossipCtx, space: SpaceId) -> BTreeSet<PublicKey> {
    let mut bootstrap = BTreeSet::new();

    for registry in ctx.followed.clients() {
        let occupants = match registry.occupants(space.namespace()).await {
            Ok(occupants) => occupants,
            Err(err) => {
                warn!(?err, "Failed querying registry presence");
                continue;
            }
        };

        // A registry holds presence with a short TTL, so this list routinely
        // contains endpoints from processes that have already exited.
        let listed = occupants
            .into_iter()
            .map(|presence| presence.endpoint)
            .filter(|endpoint| *endpoint != ctx.endpoint.id());
        bootstrap.extend(listed);
    }

    debug!(
        me = %ctx.endpoint.id().fmt_short(),
        occupants = bootstrap.len(),
        "Registry presence for space"
    );
    bootstrap
}
