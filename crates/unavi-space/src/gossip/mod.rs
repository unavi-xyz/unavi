use std::sync::Arc;

use bevy::prelude::*;
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router::{RouterBuilderFn, RouterBuilderFnTarget},
};
use bevy_wds::{LocalActor, SyncTargets};
use iroh::EndpointId;
use iroh_gossip::Gossip;
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;
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
    Presence,
    Unknown(u64),
}

#[derive(Component)]
pub struct IrohGossip(Gossip);

#[derive(Component)]
pub struct GossipSender(tokio::sync::mpsc::Sender<thread::GossipCommand>);

pub fn spawn_gossip(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    mut commands: Commands,
) {
    let endpoint = endpoints
        .get(trigger.entity)
        .map(|e| e.0.clone())
        .expect("endpoint");

    let (gossip_tx, gossip_rx) = tokio::sync::oneshot::channel();
    let (tx, rx) = tokio::sync::mpsc::channel(16);

    unavi_wasm_compat::spawn_thread(async move {
        let gossip = Gossip::builder().spawn(endpoint);
        gossip_tx.send(gossip).expect("send gossip");

        thread::handle_gossip_thread(rx).await;
    });

    let gossip = gossip_rx.blocking_recv().expect("recv gossip");

    commands
        .entity(trigger.entity)
        .insert((IrohGossip(gossip.clone()), GossipSender(tx)));

    commands.spawn((
        RouterBuilderFnTarget(trigger.entity),
        RouterBuilderFn(Some(Box::new(|router| {
            router.accept(iroh_gossip::ALPN, gossip)
        }))),
    ));
}

#[derive(Component)]
pub struct SpaceGossipCancel(Arc<Notify>);

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

    let cancel = Arc::new(Notify::default());
    let space = spaces.get(trigger.entity).map(|s| s.0).expect("space");

    commands
        .entity(trigger.entity)
        .insert(SpaceGossipCancel(Arc::clone(&cancel)));

    let _ = sender
        .0
        .blocking_send(GossipCommand::JoinSpace { ctx, cancel, space });
}

pub fn leave_space_topic(
    trigger: On<Remove, Space>,
    cancels: Query<&SpaceGossipCancel>,
    mut commands: Commands,
) {
    let Ok(cancel) = cancels.get(trigger.entity) else {
        return;
    };

    cancel.0.notify_one();

    commands
        .entity(trigger.entity)
        .remove::<SpaceGossipCancel>();
}
