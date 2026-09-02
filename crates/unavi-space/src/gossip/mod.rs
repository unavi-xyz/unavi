use bevy::prelude::*;
use bevy_iroh::{
    endpoint::IrohEndpoint,
    store::LocalStore,
};
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
use tokio::sync::{
    oneshot,
    watch,
};
use unavi_identity::signed_bytes::Signable;
use unavi_policy::space::Space;
use unavi_util::async_task::spawn_async_task;

use crate::{
    gossip::thread::{
        GossipCommand,
        GossipCtx,
    },
    peer::presence::PresenceInbox,
};

mod bootstrap;
mod inbound;
mod outbound;
mod thread;

/// The occupied space, mirrored from [`crate::anchor::ActiveSpace`] for the
/// async gossip tasks. Presence broadcasts only to this space.
///
/// A send wakes the outbound task, so entering a space broadcasts immediately
/// instead of waiting out the heartbeat.
#[derive(Resource)]
pub struct ActiveSpaceSignal(watch::Sender<Option<NamespaceId>>);

impl Default for ActiveSpaceSignal {
    fn default() -> Self {
        Self(watch::channel(None).0)
    }
}

pub fn publish_active_space(
    active: Res<crate::anchor::ActiveSpace>,
    spaces: Query<&Space>,
    signal: Res<ActiveSpaceSignal>,
) {
    if !active.is_changed() {
        return;
    }
    let hash = active.0.and_then(|e| spaces.get(e).ok()).map(|s| s.0);
    signal.0.send_if_modified(|current| {
        let changed = *current != hash;
        *current = hash;
        changed
    });
}

#[derive(Serialize, Deserialize)]
struct SpaceBroadcast {
    sender: EndpointId,
    msg:    SpaceMessage,
}

impl Signable for SpaceBroadcast {
    const SIGNING_CONTEXT: &'static str = "wired/space/broadcast";
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

/// Adopts the data store's gossip rather than spawning one: `iroh_gossip::ALPN`
/// can be accepted once per router, and the loser of that race silently drops
/// its presence broadcasts.
pub fn adopt_gossip(
    endpoints: Query<Entity, (With<IrohEndpoint>, Without<IrohGossip>)>,
    stores: Query<&LocalStore>,
    mut commands: Commands,
) {
    let Ok(store) = stores.single() else {
        return;
    };

    for entity in &endpoints {
        commands
            .entity(entity)
            .insert(IrohGossip(store.0.gossip().clone()));
    }
}

/// Subscribes every space that is not subscribed yet. A pass rather than an
/// observer, since gossip is built asynchronously; an unsubscribed space
/// broadcasts nothing and hears nothing.
pub fn join_space_topics(
    spaces: Query<(Entity, &Space), Without<SpaceGossipCancel>>,
    sender: Query<&GossipSender>,
    endpoints: Query<(&IrohEndpoint, &IrohGossip)>,
    signal: Res<ActiveSpaceSignal>,
    presence: Res<PresenceInbox>,
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
            active:   signal.0.subscribe(),
            presence: presence.inbox(),
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
    // Removing SpaceGossipCancel drops the oneshot::Sender, signalling the task
    // to cancel. Despawning the space drops it just as well, so a missing
    // entity is not an error.
    commands
        .entity(trigger.entity)
        .try_remove::<SpaceGossipCancel>();
}
