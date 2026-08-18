use std::sync::{
    LazyLock,
    RwLock,
};

use bevy::prelude::*;
use bevy_iroh::endpoint::IrohEndpoint;
use bevy_wds::LocalGossip;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_docs::NamespaceId;
use iroh_gossip::Gossip;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::oneshot;
use unavi_policy::space::Space;
use unavi_util::async_task::spawn_async_task;
use wds::signed_bytes::Signable;

use crate::gossip::thread::{
    GossipCommand,
    GossipCtx,
};

mod bootstrap;
mod inbound;
mod outbound;
mod thread;

/// The occupied space, mirrored from [`crate::anchor::ActiveSpace`] for the
/// async gossip tasks. Presence broadcasts only to this space.
static ACTIVE_SPACE: RwLock<Option<NamespaceId>> = RwLock::new(None);

/// Woken when the active space changes, so the entered space broadcasts
/// presence immediately instead of waiting out the heartbeat.
static ACTIVE_CHANGED: LazyLock<tokio::sync::Notify> = LazyLock::new(tokio::sync::Notify::new);

fn active_space() -> Option<NamespaceId> {
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

impl Signable for SpaceBroadcast {
    const SIGNING_CONTEXT: &'static str = "unavi/space/broadcast";
}

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

pub fn spawn_gossip(trigger: On<Add, IrohEndpoint>, mut commands: Commands) {
    let (tx, rx) = async_channel::bounded(32);

    spawn_async_task(async move {
        thread::handle_gossip_thread(rx).await;
    });

    commands.entity(trigger.entity).insert(GossipSender(tx));
}

/// Adopts the data store's gossip rather than spawning one.
///
/// `iroh_gossip::ALPN` can be accepted once per router. A second instance
/// would capture inbound connections, silently dropping the loser's presence
/// broadcasts while iroh-docs keeps working on the winner.
pub fn adopt_gossip(
    endpoints: Query<Entity, (With<IrohEndpoint>, Without<IrohGossip>)>,
    stores: Query<&LocalGossip>,
    mut commands: Commands,
) {
    let Ok(gossip) = stores.single() else {
        return;
    };

    for entity in &endpoints {
        commands.entity(entity).insert(IrohGossip(gossip.0.clone()));
    }
}

/// Subscribes every space that is not subscribed yet.
///
/// A pass rather than an observer: gossip is built asynchronously, so a space
/// entered before it is ready would be dropped by a hook that fires once. An
/// unsubscribed space broadcasts nothing and hears nothing.
pub fn join_space_topics(
    spaces: Query<(Entity, &Space), Without<SpaceGossipCancel>>,
    sender: Query<&GossipSender>,
    endpoints: Query<(&IrohEndpoint, &IrohGossip)>,
    mut commands: Commands,
) {
    if spaces.is_empty() {
        return;
    }

    let Ok(sender) = sender.single() else {
        return;
    };
    let Ok((endpoint, gossip)) = endpoints.single() else {
        return;
    };

    for (entity, space) in &spaces {
        let ctx = GossipCtx {
            endpoint: endpoint.0.clone(),
            gossip:   gossip.0.clone(),
        };

        let (cancel_tx, cancel_rx) = oneshot::channel();
        let space = space.0;

        commands
            .entity(entity)
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
}

#[derive(Component)]
pub struct SpaceGossipCancel {
    _cancel: oneshot::Sender<()>,
}

pub fn leave_space_topic(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing SpaceGossipCancel drops the oneshot::Sender, signalling the task to
    // cancel. Despawning the space drops it just as well, so a missing entity is
    // not an error.
    commands
        .entity(trigger.entity)
        .try_remove::<SpaceGossipCancel>();
}
