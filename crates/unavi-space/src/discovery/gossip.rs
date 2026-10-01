//! Each space's gossip topic: joined while the space is loaded (and not just
//! peeked through a portal), left when it unloads.
//!
//! Peers broadcast signed presence on the topic of the space they stand in,
//! naming the topic and when they spoke, so a broadcast cannot be replayed into
//! another space or kept alive past its time.

use std::{
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
    time::Duration,
};

use bevy::prelude::*;
use bevy_async::task;
use bevy_iroh::{
    endpoint::IrohEndpoint,
    store::DataStore,
};
use iroh::{
    Endpoint,
    EndpointAddr,
    EndpointId,
};
use iroh_gossip::{
    Gossip,
    TopicId,
    api::{
        GossipSender,
        JoinOptions,
    },
};
use n0_future::task::AbortOnDropHandle;
use rand::seq::SliceRandom;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::{
    oneshot,
    watch,
};
use tracing::Instrument;
use unavi_identity::signed::Signable;
use unavi_registry::client::Followed;
use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

use crate::{
    discovery::{
        HeardPresence,
        PRESENCE_INTERVAL,
        gossip_wire,
        registry,
    },
    grid::ActiveSpace,
    identity::LocalIdentity,
    link::inbox::Inbox,
    membership::{
        Space,
        SpaceId,
    },
    portal::Peeked,
};

/// The occupied space, mirrored from [`ActiveSpace`] for the gossip tasks.
/// Presence broadcasts only to this space.
///
/// A send wakes the outbound task, so entering a space broadcasts immediately
/// instead of waiting out the heartbeat.
#[derive(Resource)]
pub struct ActiveSpaceSignal(watch::Sender<Option<SpaceId>>);

impl Default for ActiveSpaceSignal {
    fn default() -> Self {
        Self(watch::channel(None).0)
    }
}

pub fn publish_active_space(
    active: Res<ActiveSpace>,
    spaces: Query<&Space>,
    signal: Res<ActiveSpaceSignal>,
) {
    if !active.is_changed() {
        return;
    }
    let id = active.0.and_then(|e| spaces.get(e).ok()).map(Space::id);
    signal.0.send_if_modified(|current| {
        let changed = *current != id;
        *current = id;
        changed
    });
}

#[derive(Serialize, Deserialize)]
pub(super) struct SpaceBroadcast {
    pub sender:    EndpointId,
    /// The topic's space, so a broadcast cannot be replayed into another.
    pub space:     SpaceId,
    /// Unix seconds when it was signed.
    pub issued_at: u64,
    pub msg:       SpaceMessage,
}

impl Signable for SpaceBroadcast {
    const SIGNING_CONTEXT: &'static str = "wired/space/broadcast";
}

#[derive(Serialize, Deserialize)]
pub(super) enum SpaceMessage {
    Presence(EndpointAddr),
}

/// How old a broadcast may be before it is ignored.
const MAX_BROADCAST_AGE: Duration = PRESENCE_INTERVAL.saturating_mul(2);

/// How far ahead of local time a broadcast may be stamped.
const MAX_BROADCAST_SKEW: Duration = Duration::from_secs(30);

pub(super) fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl SpaceBroadcast {
    /// Whether this broadcast was signed for `space` recently enough.
    pub(super) fn is_current(&self, space: SpaceId, now: u64) -> bool {
        self.space == space
            && self.issued_at.saturating_add(MAX_BROADCAST_AGE.as_secs()) >= now
            && self.issued_at <= now.saturating_add(MAX_BROADCAST_SKEW.as_secs())
    }
}

/// The node's gossip, borrowed from the data store.
#[derive(Component)]
pub struct IrohGossip(Gossip);

/// Adopts the data store's gossip rather than spawning one: `iroh_gossip::ALPN`
/// can be accepted once per router, and the loser of that race silently drops
/// its presence broadcasts.
pub fn adopt_gossip(
    endpoints: Query<Entity, (With<IrohEndpoint>, Without<IrohGossip>)>,
    store: Option<Res<DataStore>>,
    mut commands: Commands,
) {
    let Some(store) = store else {
        return;
    };

    for entity in &endpoints {
        commands
            .entity(entity)
            .insert(IrohGossip(store.0.gossip().clone()));
    }
}

/// What a topic's tasks share.
#[derive(Clone)]
pub(super) struct GossipCtx {
    pub endpoint: Endpoint,
    pub gossip:   Gossip,
    /// The occupied space. Only it broadcasts presence.
    pub active:   watch::Receiver<Option<SpaceId>>,
    pub presence: Inbox<(EndpointId, SpaceId), EndpointAddr>,
    /// Asked who is in a space before its gossip topic has neighbors.
    pub followed: Followed,
}

/// Held on a [`Space`] subscribed to its topic. Dropping it leaves the topic.
#[derive(Component)]
pub struct SpaceGossip {
    _leave: oneshot::Sender<()>,
}

/// Subscribes every loaded space that is not subscribed yet. A peeked space
/// joins only once it becomes the active one. A pass rather than an observer,
/// since gossip is built asynchronously.
pub fn join_space_topics(
    spaces: Query<(Entity, &Space, Has<Peeked>), Without<SpaceGossip>>,
    endpoints: Query<(&IrohEndpoint, &IrohGossip)>,
    active: Res<ActiveSpace>,
    signal: Res<ActiveSpaceSignal>,
    heard: Res<HeardPresence>,
    identity: Option<Res<LocalIdentity>>,
    mut commands: Commands,
) {
    if spaces.is_empty() {
        return;
    }
    let Some(identity) = identity else {
        return;
    };
    let Ok((endpoint, gossip)) = endpoints.single() else {
        return;
    };

    for (entity, space, peeked) in &spaces {
        if peeked && active.0 != Some(entity) {
            continue;
        }

        let ctx = GossipCtx {
            endpoint: endpoint.0.clone(),
            gossip:   gossip.0.clone(),
            active:   signal.0.subscribe(),
            presence: heard.inbox().clone(),
            followed: identity.followed.clone(),
        };
        let (leave_tx, leave_rx) = oneshot::channel();
        commands
            .entity(entity)
            .insert(SpaceGossip { _leave: leave_tx });

        let space = space.id();
        task::spawn(
            async move {
                tokio::select! {
                    _ = leave_rx => {}
                    res = handle_space_topic(ctx, space) => {
                        if let Err(err) = res {
                            error!(?err, "error handling space topic");
                        }
                    }
                }
            }
            .instrument(info_span!("gossip", %space)),
        );
    }
}

/// Leaves the topic: removing [`SpaceGossip`] drops the sender its task waits
/// on. Despawning the space drops it just as well, so a missing entity is not
/// an error.
pub fn leave_space_topic(trigger: On<Remove, Space>, mut commands: Commands) {
    commands.entity(trigger.entity).try_remove::<SpaceGossip>();
}

/// Separates this crate's per-space gossip from iroh-docs'.
///
/// iroh-docs subscribes to a namespace's own bytes as its sync topic; sharing
/// that topic would deliver each protocol the other's frames, and iroh-docs
/// permanently drops a namespace's sync on the first frame it cannot decode.
const TOPIC_CONTEXT: &str = "unavi/space/gossip";

fn space_topic(space: SpaceId) -> TopicId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TOPIC_CONTEXT.as_bytes());
    hasher.update(&[0]);
    hasher.update(&space.0);
    TopicId::from_bytes(*hasher.finalize().as_bytes())
}

/// How often a topic with no neighbors looks for bootstrap peers again.
const BOOTSTRAP_RETRY: Duration = Duration::from_secs(5);

/// How often a topic that already has neighbors re-samples the registry, to
/// find peers whose cluster this one has never met.
const BOOTSTRAP_SHUFFLE: Duration = Duration::from_mins(1);

/// How many occupants a shuffle dials.
const SHUFFLE_SAMPLE: usize = 4;

/// Keeps looking for someone to gossip with. Presence is discovered
/// asynchronously, so resolving only at join would leave the first arrival
/// broadcasting into an empty topic for the life of the process.
async fn bootstrap_loop(
    ctx: &GossipCtx,
    tx: &GossipSender,
    space: SpaceId,
    neighbors: &AtomicUsize,
) {
    let mut waited = Duration::ZERO;

    loop {
        n0_future::time::sleep(BOOTSTRAP_RETRY).await;
        waited += BOOTSTRAP_RETRY;

        // Separate clusters of peers are mutually invisible and gossip cannot
        // merge them; re-sampling the registry on a slow interval is what a
        // partitioned overlay heals by.
        let connected = neighbors.load(Ordering::Relaxed) > 0;
        if connected {
            if waited < BOOTSTRAP_SHUFFLE {
                continue;
            }
            waited = Duration::ZERO;
        }

        let mut peers = registry::find_bootstrap_peers(ctx, space)
            .await
            .into_iter()
            .collect::<Vec<_>>();
        if peers.is_empty() {
            continue;
        }

        // Dialing every occupant fans each arrival out to the whole space; a
        // random handful stitches clusters together.
        if connected && peers.len() > SHUFFLE_SAMPLE {
            peers.shuffle(&mut rand::rng());
            peers.truncate(SHUFFLE_SAMPLE);
        }

        info!(
            me = %ctx.endpoint.id().fmt_short(),
            peers = ?peers.iter().map(|p| p.fmt_short().to_string()).collect::<Vec<_>>(),
            shuffle = connected,
            "Gossip bootstrap"
        );
        if let Err(err) = tx.join_peers(peers).await {
            warn!(?err, "Failed to join bootstrap peers");
        }
    }
}

/// Runs one space's topic until a task fails or the caller drops this future,
/// which aborts every task it spawned.
async fn handle_space_topic(ctx: GossipCtx, space: SpaceId) -> anyhow::Result<()> {
    let peers = registry::find_bootstrap_peers(&ctx, space).await;
    if peers.is_empty() {
        warn!("No bootstrap peers for space topic; presence reaches nobody until one is found");
    }

    let topic = ctx
        .gossip
        .subscribe_with_opts(
            space_topic(space),
            JoinOptions {
                bootstrap:             peers,
                subscription_capacity: 256,
            },
        )
        .await?;
    let (tx, mut rx) = topic.split();

    let ctx = Arc::new(ctx);
    let wake = Arc::new(tokio::sync::Notify::new());
    let neighbors = Arc::new(AtomicUsize::new(0));

    let inbound = AbortOnDropHandle::new(n0_future::task::spawn({
        let ctx = Arc::clone(&ctx);
        let wake = Arc::clone(&wake);
        let neighbors = Arc::clone(&neighbors);
        async move {
            if let Err(err) = gossip_wire::receive(&ctx, &mut rx, space, &wake, &neighbors).await {
                error!(?err, "Error handling inbound gossip");
            }
        }
    }));

    let _bootstrap = AbortOnDropHandle::new(n0_future::task::spawn({
        let ctx = Arc::clone(&ctx);
        let tx = tx.clone();
        let neighbors = Arc::clone(&neighbors);
        async move { bootstrap_loop(&ctx, &tx, space, &neighbors).await }
    }));

    let outbound = AbortOnDropHandle::new(n0_future::task::spawn(async move {
        while let Err(err) = gossip_wire::announce(&ctx, &tx, space, &wake).await {
            error!(?err, "Error handling outbound gossip");
            n0_future::time::sleep(Duration::from_secs(1)).await;
        }
    }));

    tokio::select! {
        res = inbound => res?,
        res = outbound => res?,
    }
    Ok(())
}
