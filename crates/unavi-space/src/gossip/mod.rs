use std::sync::{
    LazyLock,
    RwLock,
};

use bevy::prelude::*;
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router::{
        RouterBuilderFn,
        RouterBuilderFnTarget,
    },
};
use bevy_wds::{
    LocalActor,
    SyncTargets,
};
use blake3::Hash;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_gossip::Gossip;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::signed_bytes::Signable;

use crate::{
    Space,
    gossip::thread::{
        GossipCommand,
        GossipCtx,
    },
};

mod bootstrap;
mod inbound;
mod outbound;
mod thread;

/// The space we currently occupy, mirrored from [`crate::anchor::ActiveSpace`]
/// for the async gossip tasks. We only broadcast presence to this space, while
/// still receiving on every space we have loaded.
static ACTIVE_SPACE: RwLock<Option<Hash>> = RwLock::new(None);

/// Woken whenever the active space changes, so the outbound task for the space
/// we just entered broadcasts presence immediately rather than waiting out the
/// heartbeat interval.
static ACTIVE_CHANGED: LazyLock<tokio::sync::Notify> = LazyLock::new(tokio::sync::Notify::new);

fn active_space() -> Option<Hash> {
    *ACTIVE_SPACE.read().expect("active space poisoned")
}

fn active_changed() -> &'static tokio::sync::Notify {
    &ACTIVE_CHANGED
}

pub fn publish_active_space(active: Res<crate::anchor::ActiveSpace>, spaces: Query<&Space>) {
    if !active.is_changed() {
        return;
    }
    let hash = active.0.and_then(|e| spaces.get(e).ok()).map(|s| s.0);
    let mut current = ACTIVE_SPACE.write().expect("active space poisoned");
    if *current == hash {
        return;
    }
    *current = hash;
    drop(current);
    ACTIVE_CHANGED.notify_waiters();
}

#[derive(Serialize, Deserialize)]
struct SpaceBroadcast {
    sender: EndpointId,
    msg:    SpaceMessage,
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
    let (tx, rx) = async_channel::bounded(32);

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
        endpoint:     endpoint.0.clone(),
        gossip:       gossip.0.clone(),
        actor:        actor.0.clone(),
        sync_targets: sync_targets.0.clone(),
    };

    let (cancel_tx, cancel_rx) = oneshot::channel();
    let space = spaces.get(trigger.entity).map(|s| s.0).expect("space");

    commands
        .entity(trigger.entity)
        .insert(SpaceGossipCancel { _cancel: cancel_tx });

    let sender = sender.0.clone();

    unavi_util::async_task::spawn_async_task(async move {
        if let Err(err) = sender
            .send(GossipCommand::JoinSpace {
                ctx,
                cancel: cancel_rx,
                space,
            })
            .await
        {
            error!(?err, "Failed to send gossip command");
        }
    });
}

#[derive(Component)]
pub struct SpaceGossipCancel {
    _cancel: oneshot::Sender<()>,
}

pub fn leave_space_topic(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing SpaceGossipCancel drops the oneshot::Sender, signalling the task to
    // cancel.
    commands
        .entity(trigger.entity)
        .remove::<SpaceGossipCancel>();
}
