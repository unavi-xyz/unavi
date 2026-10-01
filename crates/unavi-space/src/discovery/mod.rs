//! Finding peers: signed presence heard over each space's gossip topic, and
//! registry occupancy, turned into a bounded set of [`Peer`] entities.

use std::{
    sync::Arc,
    time::Duration,
};

use bevy::{
    platform::collections::{
        HashMap,
        HashSet,
    },
    prelude::*,
};
use bevy_iroh::store::BlobProviders;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use parking_lot::Mutex;
use tokio::sync::Notify;

use crate::{
    index::{
        self,
        Index,
        Indexed,
    },
    link::inbox::Inbox,
    membership::SpaceId,
};

pub mod gossip;
mod gossip_wire;
pub mod registry;

/// How often presence is re-broadcast on a space's topic.
pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(20);

/// How long a peer stays in a space after it was last heard there.
const INACTIVE: Duration = PRESENCE_INTERVAL.saturating_mul(4);

/// Most peers tracked at once. Presence is free to sign, so this is what
/// bounds the entities and dials a stranger can cause.
pub const MAX_PEERS: usize = 256;

/// Most peers tracked in one space.
pub const MAX_PEERS_PER_SPACE: usize = 64;

/// A peer heard in some space, dialed while it is tracked.
#[derive(Component)]
#[component(on_insert = index::insert::<Self>, on_discard = index::discard::<Self>)]
#[require(LastSeenIn)]
pub struct Peer(pub EndpointAddr);

impl Indexed for Peer {
    type Key = EndpointId;

    fn key(&self) -> EndpointId {
        self.0.id
    }
}

/// When a [`Peer`] was last heard in each space.
#[derive(Component, Default)]
pub struct LastSeenIn(HashMap<SpaceId, Duration>);

impl LastSeenIn {
    #[must_use]
    pub fn contains(&self, space: SpaceId) -> bool {
        self.0.contains_key(&space)
    }

    pub fn spaces(&self) -> impl Iterator<Item = SpaceId> + '_ {
        self.0.keys().copied()
    }

    /// Marks the peer as still in `space`, if it was already there.
    pub fn refresh(&mut self, space: SpaceId, now: Duration) {
        if let Some(at) = self.0.get_mut(&space) {
            *at = now;
        }
    }
}

/// Presence broadcasts heard over gossip, handed from each topic's inbound
/// task to [`track_peers`].
#[derive(Resource, Clone, Default)]
pub struct HeardPresence(Inbox<(EndpointId, SpaceId), EndpointAddr>);

impl HeardPresence {
    #[must_use]
    pub const fn inbox(&self) -> &Inbox<(EndpointId, SpaceId), EndpointAddr> {
        &self.0
    }
}

/// Which space each tracked peer is present in, readable from network tasks.
/// Mirrors [`LastSeenIn`].
#[derive(Resource, Clone, Default)]
pub struct PeerPresence(Arc<PresenceInner>);

#[derive(Default)]
struct PresenceInner {
    spaces:  Mutex<HashMap<EndpointId, HashSet<SpaceId>>>,
    changed: Notify,
}

impl PeerPresence {
    #[must_use]
    pub fn is_in(&self, peer: EndpointId, space: SpaceId) -> bool {
        self.0
            .spaces
            .lock()
            .get(&peer)
            .is_some_and(|spaces| spaces.contains(&space))
    }

    /// Whether `peer` is in `space`, waiting up to `deadline` for presence
    /// still on its way.
    pub async fn wait(&self, peer: EndpointId, space: SpaceId, deadline: Duration) -> bool {
        let wait = async {
            loop {
                let mut changed = std::pin::pin!(self.0.changed.notified());
                changed.as_mut().enable();
                if self.is_in(peer, space) {
                    return;
                }
                changed.await;
            }
        };
        n0_future::time::timeout(deadline, wait).await.is_ok()
    }

    fn replace(&self, spaces: HashMap<EndpointId, HashSet<SpaceId>>) {
        *self.0.spaces.lock() = spaces;
        self.0.changed.notify_waiters();
    }
}

/// Turns heard presence into [`Peer`] entities, within [`MAX_PEERS`] and
/// [`MAX_PEERS_PER_SPACE`], and expires peers not heard for a while.
pub fn track_peers(
    time: Res<Time>,
    heard: Res<HeardPresence>,
    index: Res<Index<Peer>>,
    presence: Res<PeerPresence>,
    mut peers: Query<(Entity, &mut Peer, &mut LastSeenIn)>,
    mut commands: Commands,
) {
    let now = time.elapsed();
    let updates = heard.0.drain();
    let mut changed = !updates.is_empty();

    let mut per_space = HashMap::<SpaceId, usize>::default();
    for (_, _, seen) in &peers {
        for space in seen.spaces() {
            *per_space.entry(space).or_default() += 1;
        }
    }
    let mut total = index.len();
    let mut spawned = HashMap::<EndpointId, SpaceId>::default();

    for ((id, space), addr) in updates {
        let room = per_space.get(&space).copied().unwrap_or_default() < MAX_PEERS_PER_SPACE;

        if let Some(entity) = index.get(id)
            && let Ok((_, mut peer, mut seen)) = peers.get_mut(entity)
        {
            if seen.contains(space) {
                seen.refresh(space, now);
            } else if room {
                seen.0.insert(space, now);
                *per_space.entry(space).or_default() += 1;
            }
            if peer.0 != addr {
                peer.0 = addr;
            }
            continue;
        }

        if spawned.contains_key(&id) {
            continue;
        }
        if total >= MAX_PEERS || !room {
            debug!(peer = %id, %space, "peer cap reached; presence ignored");
            continue;
        }
        info!("+peer: {id}");
        let mut seen = LastSeenIn::default();
        seen.0.insert(space, now);
        commands.spawn((Peer(addr), seen));
        *per_space.entry(space).or_default() += 1;
        total += 1;
        spawned.insert(id, space);
    }

    let limit = now.saturating_sub(INACTIVE);
    for (entity, peer, mut seen) in &mut peers {
        let before = seen.0.len();
        seen.0.retain(|_, at| *at > limit);
        if seen.0.len() != before {
            changed = true;
        }
        if seen.0.is_empty() {
            info!("-peer: {}", peer.0.id);
            commands.entity(entity).despawn();
        }
    }

    if changed {
        let mut spaces = peers
            .iter()
            .filter(|(_, _, seen)| !seen.0.is_empty())
            .map(|(_, peer, seen)| (peer.0.id, seen.spaces().collect::<HashSet<_>>()))
            .collect::<HashMap<_, _>>();
        for (id, space) in spawned {
            spaces.entry(id).or_default().insert(space);
        }
        presence.replace(spaces);
    }
}

/// Offers the tracked peers to the blob downloader.
///
/// A space's document syncs from its occupants, so its content lives with them
/// too; a fetch knowing only the configured sync targets asks a server that may
/// never have seen the space.
pub fn publish_blob_providers(peers: Query<&Peer>, providers: Option<ResMut<BlobProviders>>) {
    let Some(mut providers) = providers else {
        return;
    };
    let connected = peers.iter().map(|p| p.0.id).collect::<Vec<_>>();
    if providers.0 != connected {
        providers.0 = connected;
    }
}
