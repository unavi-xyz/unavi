use bevy::prelude::*;
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router::{RouterBuilderFn, RouterBuilderFnTarget},
};
use bevy_wds::{LocalActor, SyncTargets};
use iroh::{EndpointAddr, EndpointId};
use iroh_gossip::Gossip;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::signed_bytes::Signable;

use crate::{
    Space,
    gossip::thread::{GossipCommand, GossipCtx},
};

mod bootstrap;
mod inbound;
mod outbound;
mod thread;

#[derive(Serialize, Deserialize)]
struct SpaceBroadcast {
    sender: EndpointId,
    msg: SpaceMessage,
}

impl Signable for SpaceBroadcast {}

#[derive(Serialize, Deserialize)]
#[non_exhaustive]
enum SpaceMessage {
    Presence(EndpointAddr),
    Unknown(usize),
}

#[derive(Component)]
pub struct IrohGossip(Gossip);

#[derive(Component)]
pub struct GossipSender(async_channel::Sender<thread::GossipCommand>);

#[derive(Component)]
pub struct PendingGossip(async_channel::Receiver<Gossip>);

pub fn spawn_gossip(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    mut commands: Commands,
) {
    let endpoint = endpoints
        .get(trigger.entity)
        .map(|e| e.0.clone())
        .expect("endpoint");

    let (gossip_tx, gossip_rx) = async_channel::bounded(1);
    let (tx, rx) = async_channel::bounded(16);

    spawn_async_task(async move {
        let gossip = Gossip::builder().spawn(endpoint);
        gossip_tx.send(gossip).await.ok();
        thread::handle_gossip_thread(rx).await;
    });

    commands
        .entity(trigger.entity)
        .insert((GossipSender(tx), PendingGossip(gossip_rx)));
}

pub fn poll_gossip(pending: Query<(Entity, &PendingGossip)>, mut commands: Commands) {
    for (entity, p) in &pending {
        let Ok(gossip) = p.0.try_recv() else {
            continue;
        };

        commands
            .entity(entity)
            .insert(IrohGossip(gossip.clone()))
            .remove::<PendingGossip>();

        commands.spawn((
            RouterBuilderFnTarget(entity),
            RouterBuilderFn(Some(Box::new(|router| {
                router.accept(iroh_gossip::ALPN, gossip)
            }))),
        ));
    }
}

pub fn join_space_topic(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    sender: Query<&GossipSender>,
    endpoints: Query<(&IrohEndpoint, &IrohGossip)>,
    actors: Query<(&LocalActor, &SyncTargets)>,
    mut commands: Commands,
) {
    let Ok(sender) = sender.single() else {
        warn!("Cannot join space topic: no gossip sender");
        return;
    };

    let Ok((endpoint, gossip)) = endpoints.single() else {
        warn!("Space add failed: no endpoint");
        return;
    };

    let Ok((actor, sync_targets)) = actors.single() else {
        warn!("Space add failed: no actor");
        return;
    };

    let ctx = GossipCtx {
        endpoint: endpoint.0.clone(),
        gossip: gossip.0.clone(),
        actor: actor.0.clone(),
        sync_targets: sync_targets.0.clone(),
    };

    let (cancel_tx, cancel_rx) = oneshot::channel();
    let space = spaces.get(trigger.entity).map(|s| s.0).expect("space");

    commands
        .entity(trigger.entity)
        .insert(SpaceGossipCancel { _cancel: cancel_tx });

    sender
        .0
        .send_blocking(GossipCommand::JoinSpace {
            ctx,
            cancel: cancel_rx,
            space,
        })
        .expect("send gossip command");
}

#[derive(Component)]
pub struct SpaceGossipCancel {
    _cancel: oneshot::Sender<()>,
}

pub fn leave_space_topic(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing SpaceGossipCancel drops the oneshot::Sender, signalling the task to cancel.
    commands
        .entity(trigger.entity)
        .remove::<SpaceGossipCancel>();
}
