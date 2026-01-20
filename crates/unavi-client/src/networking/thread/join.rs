use std::{collections::HashSet, time::Duration};

use bevy::{
    ecs::world::CommandQueue,
    log::{debug, error, info, warn},
    tasks::futures_lite::StreamExt,
};
use blake3::Hash;
use iroh::{EndpointId, Signature};
use iroh_gossip::{
    TopicId,
    api::{Event, GossipReceiver, GossipSender},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use wds::signed_bytes::{IrohSigner, Signable, SignedBytes};
use wired_schemas::SCHEMA_BEACON;

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    networking::thread::NetworkThreadState,
    space::{Space, SpaceDoc, beacon::Beacon},
};

/// Broadcast to a space gossip topic that you are joining as `endpoint`.
/// Message is signed and verified with the given endpoint ID.
#[derive(Serialize, Deserialize)]
struct JoinBroadcast {
    endpoint: EndpointId,
}

impl Signable for JoinBroadcast {}

pub async fn handle_join(state: NetworkThreadState, space_id: Hash) -> anyhow::Result<()> {
    // Download space.
    let space_doc = {
        let mut builder = state
            .local_actor
            .read(space_id)
            .ttl(Duration::from_hours(24 * 3));

        if let Some(actor) = &state.remote_actor {
            builder = builder.sync_from((*actor.host()).into());
        }

        builder.send().await?
    };

    // Query beacons to find players.
    let mut bootstrap = HashSet::new();

    if let Some(actor) = &state.remote_actor {
        let found = actor.query().schema(SCHEMA_BEACON.hash).send().await?;

        let remote_host = *actor.host();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        for id in found {
            info!(%id, "Reading beacon");

            match state
                .local_actor
                .read(id)
                .sync_from(remote_host.into())
                .send()
                .await
            {
                Ok(doc) => {
                    let Ok(beacon) = Beacon::load(&doc) else {
                        debug!("invalid beacon documnt");
                        continue;
                    };

                    // Ignore old beacons.
                    if now >= beacon.expires {
                        continue;
                    }

                    // Ignore our own beacon.
                    if beacon.endpoint == state.endpoint.id() {
                        continue;
                    }

                    bootstrap.insert(beacon.endpoint);
                }
                Err(err) => {
                    warn!(?err, "failed to sync beacon");
                }
            }
        }
    }

    info!(?bootstrap, "joining gossip topic");

    // Join gossip topic.
    let topic_id = TopicId::from_bytes(*space_id.as_bytes());
    let topic = state
        .gossip
        .subscribe(topic_id, bootstrap.into_iter().collect())
        .await?;
    let (tx, rx) = topic.split();

    // Create space in ECS.
    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([(
        Space(space_id),
        SpaceDoc(space_doc),
    )]));
    ASYNC_COMMAND_QUEUE.0.send(commands).await?;

    handle_gossip_inbound(state, tx, rx).await?;

    Ok(())
}

async fn handle_gossip_inbound(
    state: NetworkThreadState,
    tx: GossipSender,
    mut rx: GossipReceiver,
) -> anyhow::Result<()> {
    while let Some(event) = rx.next().await {
        match event? {
            Event::NeighborUp(n) => {
                info!("+neighbor: {n}");
                // Broadcast join whenever we gain a new neighbor.
                // New neighbors may mean new enclaves of peers to discover.
                let join = JoinBroadcast {
                    endpoint: state.endpoint.id(),
                };
                let signed_join = join.sign(&IrohSigner(state.endpoint.secret_key()))?;
                let bytes = postcard::to_stdvec(&signed_join)?;
                tx.broadcast(bytes.into()).await?;
            }
            Event::NeighborDown(n) => {
                info!("-neighbor: {n}");
            }
            Event::Lagged => {}
            Event::Received(msg) => {
                match postcard::from_bytes::<SignedBytes<JoinBroadcast>>(&msg.content) {
                    Ok(signed_join) => {
                        let join = signed_join.payload()?;
                        info!(player = %join.endpoint, "Got join broadcast");

                        // Verify signature.
                        let Ok(sig_bytes) = signed_join.signature().try_into() else {
                            warn!(
                                "got invalid join signature length: {}",
                                signed_join.signature().len()
                            );
                            continue;
                        };
                        let sig = Signature::from_bytes(sig_bytes);

                        if let Err(err) = join.endpoint.verify(signed_join.payload_bytes(), &sig) {
                            warn!(?err, "got invalid join signature");
                            continue;
                        }

                        // Spawn connection to player.
                        if state.outbound.get_async(&join.endpoint).await.is_some() {
                            // Already connected to player.
                            continue;
                        }

                        info!("Player joined: {}", join.endpoint);

                        // Outbound handler will register itself in state.outbound.
                        let state = state.clone();
                        let remote = join.endpoint;
                        n0_future::task::spawn(async move {
                            if let Err(err) =
                                super::space::outbound::handle_outbound(state, remote).await
                            {
                                error!(?err, "error handling space outbound");
                            }
                        });
                    }
                    Err(err) => {
                        warn!(?err, "got invalid gossip message");
                    }
                }
            }
        }
    }

    Ok(())
}
