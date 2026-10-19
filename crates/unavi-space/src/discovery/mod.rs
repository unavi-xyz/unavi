//! Finding peers: signed presence heard over each space's gossip topic, and
//! registry occupancy, turned into a bounded set of [`Peer`] entities.

use std::time::Duration;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_iroh::store::BlobProviders;
use iroh::{
    EndpointAddr,
    EndpointId,
};

use crate::{
    discovery::{
        gossip::{
            MAX_BROADCAST_AGE,
            unix_secs,
        },
        stamps::Stamps,
    },
    index::{
        self,
        Index,
        Indexed,
    },
    link::inbox::Inbox,
    membership::{
        Space,
        SpaceId,
    },
    replication::Replicas,
};

pub mod gossip;
mod gossip_wire;
pub mod registry;
mod stamps;

#[cfg(test)] mod tests;

/// How often presence is re-broadcast on a space's topic.
pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(10);

/// How long a peer stays in a space after it was last heard there. Tolerates
/// two lost presence broadcasts.
const INACTIVE: Duration = PRESENCE_INTERVAL.saturating_mul(3);

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

/// What a peer last announced about a space on its gossip topic.
pub enum Heard {
    /// Carries the unix seconds the sender signed it.
    Present(EndpointAddr, u64),
    /// Carries the unix seconds the sender signed it.
    Left(u64),
}

/// Presence broadcasts heard over gossip, handed from each topic's inbound
/// task to [`track_peers`].
#[derive(Resource, Clone, Default)]
pub struct HeardPresence(Inbox<(EndpointId, SpaceId), Heard>);

impl HeardPresence {
    #[must_use]
    pub const fn inbox(&self) -> &Inbox<(EndpointId, SpaceId), Heard> {
        &self.0
    }
}

/// Turns heard presence into [`Peer`] entities, within [`MAX_PEERS`] and
/// [`MAX_PEERS_PER_SPACE`], and expires peers not heard for a while.
pub fn track_peers(
    time: Res<Time>,
    heard: Res<HeardPresence>,
    index: Res<Index<Peer>>,
    replicas: Res<Replicas>,
    mut peers: Query<(Entity, &mut Peer, &mut LastSeenIn)>,
    mut stamps: Local<Stamps>,
    mut commands: Commands,
) {
    let now = time.elapsed();
    let updates = heard.0.drain();

    let mut per_space = HashMap::<SpaceId, usize>::default();
    for (_, _, seen) in &peers {
        for space in seen.spaces() {
            *per_space.entry(space).or_default() += 1;
        }
    }
    let mut total = index.len();
    let mut spawned = HashMap::<EndpointId, SpaceId>::default();
    let mut arrived = Vec::<EndpointId>::new();

    stamps.prune(unix_secs(), MAX_BROADCAST_AGE);

    for ((id, space), heard) in updates {
        let addr = match heard {
            Heard::Present(addr, issued_at) => {
                if !stamps.admit((id, space), issued_at, false) {
                    continue;
                }
                addr
            }
            Heard::Left(issued_at) => {
                if !stamps.admit((id, space), issued_at, true) {
                    continue;
                }
                if let Some(entity) = index.get(id)
                    && let Ok((_, _, mut seen)) = peers.get_mut(entity)
                {
                    seen.0.remove(&space);
                }
                continue;
            }
        };
        let room = per_space.get(&space).copied().unwrap_or_default() < MAX_PEERS_PER_SPACE;

        if let Some(entity) = index.get(id)
            && let Ok((_, mut peer, mut seen)) = peers.get_mut(entity)
        {
            if seen.contains(space) {
                seen.refresh(space, now);
            } else if room {
                seen.0.insert(space, now);
                *per_space.entry(space).or_default() += 1;
                arrived.push(id);
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
        arrived.push(id);
    }

    // A peer that arrives in a space may have dropped what it knew of this
    // node while away, and pins are otherwise restated only when a stream
    // opens, so it is sent the local state afresh.
    arrived.sort_unstable();
    arrived.dedup();
    for id in arrived {
        replicas.resync_peer(id);
    }

    let limit = now.saturating_sub(INACTIVE);
    for (entity, peer, mut seen) in &mut peers {
        seen.0.retain(|_, at| *at >= limit);
        if seen.0.is_empty() {
            info!("-peer: {}", peer.0.id);
            commands.entity(entity).despawn();
        }
    }
}

/// Restates the local state to every peer heard in a space just loaded here.
///
/// A space reloaded while its occupants stayed connected keeps them in
/// [`LastSeenIn`], so no arrival is seen for them; what this node dropped with
/// the space is theirs to restate, and what they dropped is ours.
pub fn resync_on_space_added(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    peers: Query<(&Peer, &LastSeenIn)>,
    replicas: Res<Replicas>,
) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    for (peer, seen) in &peers {
        if seen.contains(space.id()) {
            replicas.resync_peer(peer.0.id);
        }
    }
}

/// Offers the tracked peers to the blob downloader.
///
/// A space's document syncs from its occupants, so its content lives with them
/// too; a fetch knowing only the configured sync targets asks a node that may
/// never have seen the space.
pub fn publish_blob_providers(peers: Query<&Peer>, providers: Option<ResMut<BlobProviders>>) {
    let Some(mut providers) = providers else {
        return;
    };
    let tracked = peers.iter().map(|p| p.0.id).collect::<Vec<_>>();
    if providers.0 != tracked {
        providers.0 = tracked;
    }
}
