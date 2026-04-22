use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use bevy_iroh::{IrohEndpoint, RouterBuilderFn, RouterBuilderFnTarget};
use bevy_wds::{LocalActor, SyncTargets};
use blake3::Hash;
use iroh::{Endpoint, EndpointId};
use iroh_gossip::{Gossip, TopicId, api::JoinOptions};
use serde::{Deserialize, Serialize};
use tracing::Instrument;
use wds::{
    actor::Actor,
    signed_bytes::{Signable, SignedBytes},
};

use crate::Space;

mod bootstrap;
mod inbound;
mod outbound;

#[derive(Serialize, Deserialize)]
struct SpaceBroadcast {
    sender: EndpointId,
    msg: SignedBytes<SpaceMessage>,
}

#[derive(Serialize, Deserialize)]
enum SpaceMessage {
    Presence,
}

impl Signable for SpaceMessage {}

#[derive(Component)]
pub struct IrohGossip(Gossip);

pub fn spawn_gossip(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    mut commands: Commands,
) {
    let endpoint = endpoints.get(trigger.entity).expect("endpoint");
    let gossip = Gossip::builder().spawn(endpoint.0.clone());

    commands
        .entity(trigger.entity)
        .insert(IrohGossip(gossip.clone()));

    commands.spawn((
        RouterBuilderFnTarget(trigger.entity),
        RouterBuilderFn(Some(Box::new(|router| {
            router.accept(iroh_gossip::ALPN, gossip)
        }))),
    ));
}

pub struct GossipCtx {
    endpoint: Endpoint,
    gossip: Gossip,
    actor: Actor,
    sync_targets: Vec<Actor>,
}

#[derive(Component)]
pub struct SpaceGossipCancel(Option<tokio::sync::oneshot::Sender<()>>);

pub fn on_space_add(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    endpoints: Query<(&IrohEndpoint, &IrohGossip)>,
    actors: Query<(&LocalActor, &SyncTargets)>,
    mut commands: Commands,
) {
    let space = spaces.get(trigger.entity).map(|s| s.0).expect("space");

    let Some((endpoint, gossip)) = endpoints.into_iter().next() else {
        warn!("space add failed: no endpoint");
        return;
    };

    let Some((actor, sync_targets)) = actors.into_iter().next() else {
        warn!("space add failed: no actor");
        return;
    };

    let ctx = GossipCtx {
        endpoint: endpoint.0.clone(),
        gossip: gossip.0.clone(),
        actor: actor.0.clone(),
        sync_targets: sync_targets.0.clone(),
    };

    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
    commands
        .entity(trigger.entity)
        .insert(SpaceGossipCancel(Some(cancel_tx)));

    unavi_wasm_compat::spawn_thread(async move {
        let span = info_span!("gossip", %space);

        if let Err(err) = handle_space_topic(ctx, cancel_rx, space)
            .instrument(span)
            .await
        {
            error!(?err, "error handling space topic");
        }
    });
}

async fn handle_space_topic(
    ctx: GossipCtx,
    cancel_rx: tokio::sync::oneshot::Receiver<()>,
    space: Hash,
) -> anyhow::Result<()> {
    let peers = bootstrap::find_bootstrap_peers(&ctx, space).await?;

    let topic_id = TopicId::from_bytes(*space.as_bytes());
    let topic = ctx
        .gossip
        .subscribe_with_opts(
            topic_id,
            JoinOptions {
                bootstrap: peers,
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
                match inbound::handle_gossip_inbound(&ctx, &mut rx, space).await {
                    Ok(()) => {
                        break;
                    }
                    Err(err) => {
                        error!(?err, "error handling inbound gossip");
                        n0_future::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    });

    let outbound_task = n0_future::task::spawn(async move {
        loop {
            match outbound::handle_gossip_outbound(&ctx, &tx).await {
                Ok(()) => {
                    break;
                }
                Err(err) => {
                    error!(?err, "error handling outbound gossip");
                    n0_future::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    tokio::select! {
        res = cancel_rx => {
            if let Err(err) = res {
                error!(?err, "cancel");
            }
        }
        res = inbound_task => {
            if let Err(err) = res {
                error!(?err, "inbound task");
            }
        }
        res = outbound_task => {
            if let Err(err) = res {
                error!(?err, "outbound task");
            }
        }
    }

    Ok(())
}

pub fn on_space_remove(
    trigger: On<Remove, Space>,
    mut cancels: Query<&mut SpaceGossipCancel>,
    mut commands: Commands,
) {
    let Ok(mut cancel) = cancels.get_mut(trigger.entity) else {
        return;
    };

    if let Some(cancel) = cancel.0.take() {
        let _ = cancel.send(());
    }

    commands
        .entity(trigger.entity)
        .remove::<SpaceGossipCancel>();
}
