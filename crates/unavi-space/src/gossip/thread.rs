use std::{
    sync::Arc,
    time::Duration,
};

use bevy::prelude::*;
use iroh::Endpoint;
use iroh_docs::NamespaceId;
use iroh_gossip::{
    Gossip,
    TopicId,
    api::JoinOptions,
};
use tokio::sync::oneshot;
use tracing::Instrument;

#[derive(Clone)]
pub struct GossipCtx {
    pub endpoint: Endpoint,
    pub gossip:   Gossip,
}

/// Separates this crate's per-space gossip from iroh-docs'.
///
/// iroh-docs subscribes to a namespace's own bytes as its sync topic, so
/// deriving ours the same way put two protocols on one topic: each received the
/// other's frames, and iroh-docs drops a namespace's gossip sync permanently on
/// the first frame it cannot decode. Live document updates would stop arriving
/// the moment a space broadcast anything.
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

async fn handle_space_topic(
    ctx: GossipCtx,
    cancel: oneshot::Receiver<()>,
    space: NamespaceId,
) -> anyhow::Result<()> {
    let peers = super::bootstrap::find_bootstrap_peers(&ctx, space).await?;

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

    let inbound_task = n0_future::task::spawn({
        let ctx = Arc::clone(&ctx);
        let wake = Arc::clone(&wake);
        async move {
            loop {
                match super::inbound::handle_gossip_inbound(&ctx, &mut rx, space, &wake).await {
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

    Ok(())
}
