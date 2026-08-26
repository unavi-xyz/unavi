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
use iroh::Endpoint;
use iroh_docs::NamespaceId;
use iroh_gossip::{
    Gossip,
    TopicId,
    api::{
        GossipSender,
        JoinOptions,
    },
};
use rand::seq::SliceRandom;
use tokio::sync::oneshot;
use tracing::Instrument;

#[derive(Clone)]
pub struct GossipCtx {
    pub endpoint: Endpoint,
    pub gossip:   Gossip,
}

/// Separates this crate's per-space gossip from iroh-docs'.
///
/// iroh-docs subscribes to a namespace's own bytes as its sync topic; sharing
/// that topic would deliver each protocol the other's frames, and iroh-docs
/// permanently drops a namespace's sync on the first frame it cannot decode.
const TOPIC_CONTEXT: &str = "unavi/space/gossip";

fn space_topic(space: NamespaceId) -> TopicId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TOPIC_CONTEXT.as_bytes());
    hasher.update(&[0]);
    hasher.update(space.as_bytes());
    TopicId::from_bytes(*hasher.finalize().as_bytes())
}

pub enum GossipCommand {
    JoinSpace {
        ctx:    GossipCtx,
        cancel: oneshot::Receiver<()>,
        space:  NamespaceId,
    },
}

pub async fn handle_gossip_thread(rx: async_channel::Receiver<GossipCommand>) {
    while let Ok(cmd) = rx.recv().await {
        match cmd {
            GossipCommand::JoinSpace { ctx, cancel, space } => {
                n0_future::task::spawn(async move {
                    let span = info_span!("gossip", %space);

                    if let Err(err) = handle_space_topic(ctx, cancel, space)
                        .instrument(span)
                        .await
                    {
                        error!(?err, "error handling space topic");
                    }
                });
            }
        }
    }
}

/// How often a topic with no neighbors looks for bootstrap peers again.
const BOOTSTRAP_RETRY: Duration = Duration::from_secs(5);

/// How often a topic that already has neighbors re-samples the registry, to
/// find peers whose cluster this one has never met.
const BOOTSTRAP_SHUFFLE: Duration = Duration::from_mins(1);

/// How many occupants a shuffle dials.
const SHUFFLE_SAMPLE: usize = 4;

/// Keeps looking for someone to gossip with until the topic has a neighbor.
/// Presence is discovered asynchronously, so resolving only at join would
/// leave the first arrival broadcasting into an empty topic for the life of
/// the process.
async fn handle_gossip_bootstrap(
    ctx: &GossipCtx,
    tx: &GossipSender,
    space: NamespaceId,
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

        let peers = match super::bootstrap::find_bootstrap_peers(ctx, space).await {
            Ok(peers) => peers,
            Err(err) => {
                warn!(?err, "Failed to find bootstrap peers");
                continue;
            }
        };
        if peers.is_empty() {
            continue;
        }

        let mut peers = peers.into_iter().collect::<Vec<_>>();

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

async fn handle_space_topic(
    ctx: GossipCtx,
    cancel: oneshot::Receiver<()>,
    space: NamespaceId,
) -> anyhow::Result<()> {
    let peers = super::bootstrap::find_bootstrap_peers(&ctx, space).await?;

    if peers.is_empty() {
        warn!("No bootstrap peers for space topic; presence reaches nobody until one is found");
    }

    let topic_id = space_topic(space);
    let topic = ctx
        .gossip
        .subscribe_with_opts(
            topic_id,
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

    let inbound_task = n0_future::task::spawn({
        let ctx = Arc::clone(&ctx);
        let wake = Arc::clone(&wake);
        let neighbors = Arc::clone(&neighbors);
        async move {
            loop {
                match super::inbound::handle_gossip_inbound(&ctx, &mut rx, space, &wake, &neighbors)
                    .await
                {
                    Ok(()) => {
                        break;
                    }
                    Err(err) => {
                        error!(?err, "Error handling inbound gossip");
                        n0_future::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    });

    let bootstrap_task = n0_future::task::spawn({
        let ctx = Arc::clone(&ctx);
        let tx = tx.clone();
        let neighbors = Arc::clone(&neighbors);
        async move { handle_gossip_bootstrap(&ctx, &tx, space, &neighbors).await }
    });

    let outbound_task = n0_future::task::spawn(async move {
        while let Err(err) = super::outbound::handle_gossip_outbound(&ctx, &tx, space, &wake).await
        {
            error!(?err, "Error handling outbound gossip");
            n0_future::time::sleep(Duration::from_secs(1)).await;
        }
    });

    tokio::select! {
        _ = cancel => {}
        res = inbound_task => {
            if let Err(err) = res {
                error!(?err, "Inbound task");
            }
        }
        res = outbound_task => {
            if let Err(err) = res {
                error!(?err, "Outbound task");
            }
        }
    }

    bootstrap_task.abort();

    Ok(())
}
