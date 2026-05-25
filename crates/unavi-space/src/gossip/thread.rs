use std::{
    sync::Arc,
    time::Duration,
};

use bevy::prelude::*;
use blake3::Hash;
use iroh::Endpoint;
use iroh_gossip::{
    Gossip,
    TopicId,
    api::JoinOptions,
};
use tokio::sync::oneshot;
use tracing::Instrument;
use wds::actor::Actor;

#[derive(Clone)]
pub struct GossipCtx {
    pub endpoint:     Endpoint,
    pub gossip:       Gossip,
    pub actor:        Actor,
    pub sync_targets: Vec<Actor>,
}

pub enum GossipCommand {
    JoinSpace {
        ctx:    GossipCtx,
        cancel: oneshot::Receiver<()>,
        space:  Hash,
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
    space: Hash,
) -> anyhow::Result<()> {
    let peers = super::bootstrap::find_bootstrap_peers(&ctx, space).await?;

    let topic_id = TopicId::from_bytes(*space.as_bytes());
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

    let inbound_task = n0_future::task::spawn({
        let ctx = Arc::clone(&ctx);
        async move {
            loop {
                match super::inbound::handle_gossip_inbound(&ctx, &mut rx, space).await {
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
        while let Err(err) = super::outbound::handle_gossip_outbound(&ctx, &tx).await {
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
