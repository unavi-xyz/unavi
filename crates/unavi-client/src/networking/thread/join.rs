use std::{collections::BTreeSet, sync::Arc, time::Duration};

use anyhow::bail;
use bevy::{
    ecs::world::CommandQueue,
    log::{debug, error, info, warn},
    tasks::futures_lite::StreamExt,
};
use blake3::Hash;
use iroh::Signature;
use iroh_gossip::{
    TopicId,
    api::{Event, GossipReceiver, JoinOptions},
};
use time::OffsetDateTime;
use wds::signed_bytes::{IrohSigner, Signable, SignedBytes};
use wired_schemas::SCHEMA_BEACON;

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    networking::thread::{
        NetworkEvent, NetworkThreadState,
        space::{
            SpaceHandle,
            gossip::{JoinBroadcast, SpaceGossipMsg},
            ownership::ObjectOwnership,
        },
    },
    space::{Space, SpaceDoc, beacon::Beacon},
};

pub async fn handle_join(state: NetworkThreadState, space_id: Hash) -> anyhow::Result<()> {
    // Download space.
    let space_doc = {
        let mut builder = state
            .local_actor
            .read(space_id)
            .ttl(Duration::from_hours(24 * 3));

        if let Some(remote_actor) = &state.remote_actor {
            builder = builder.sync_from(remote_actor.host().clone());
        }

        builder.send().await?
    };

    // Query beacons to find players.
    let mut bootstrap = BTreeSet::new();

    if let Some(remote_actor) = &state.remote_actor {
        let found = remote_actor
            .query()
            .schema(SCHEMA_BEACON.hash)
            .send()
            .await?;

        let now = OffsetDateTime::now_utc().unix_timestamp();

        for id in found {
            info!(%id, "Reading beacon");

            match state
                .local_actor
                .read(id)
                .sync_from(remote_actor.host().clone())
                .send()
                .await
            {
                Ok(doc) => {
                    let Ok(beacon) = Beacon::load(&doc) else {
                        debug!("invalid beacon document");
                        continue;
                    };

                    if now >= beacon.expires {
                        continue;
                    }

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
        .subscribe_with_opts(
            topic_id,
            JoinOptions {
                bootstrap,
                subscription_capacity: 256,
            },
        )
        .await?;
    let (tx, mut rx) = topic.split();

    // Register space handle.
    let ownership = Arc::new(ObjectOwnership::new());
    let handle = SpaceHandle {
        gossip_tx: tx.clone(),
        ownership,
    };
    let _ = state.spaces.insert_async(space_id, handle).await;

    // Create space in ECS.
    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([(
        Space(space_id),
        SpaceDoc(space_doc),
    )]));
    ASYNC_COMMAND_QUEUE.0.send(commands).await?;

    while let Err(err) = handle_gossip_inbound(&state, &tx, &mut rx).await {
        error!(?err, "error handling inbound gossip");
        n0_future::time::sleep(Duration::from_millis(100)).await;
    }

    // Clean up space handle on exit.
    let _ = state.spaces.remove_async(&space_id).await;

    Ok(())
}

async fn handle_gossip_inbound(
    state: &NetworkThreadState,
    tx: &iroh_gossip::api::GossipSender,
    rx: &mut GossipReceiver,
) -> anyhow::Result<()> {
    while let Some(event) = rx.next().await {
        match event? {
            Event::NeighborUp(n) => {
                info!("+neighbor: {n}");
                // Broadcast join whenever we gain a new neighbor.
                let join = SpaceGossipMsg::Join(JoinBroadcast {
                    endpoint: state.endpoint.addr(),
                });
                let signed = join.sign(&IrohSigner(state.endpoint.secret_key()))?;
                let bytes = postcard::to_stdvec(&signed)?;
                tx.broadcast(bytes.into()).await?;
            }
            Event::NeighborDown(n) => {
                info!("-neighbor: {n}");
            }
            Event::Lagged => bail!("lagged"),
            Event::Received(msg) => {
                let signed_msg =
                    match postcard::from_bytes::<SignedBytes<SpaceGossipMsg>>(&msg.content) {
                        Ok(v) => v,
                        Err(err) => {
                            warn!(?err, "got invalid gossip message");
                            continue;
                        }
                    };

                let payload = match signed_msg.payload() {
                    Ok(p) => p,
                    Err(err) => {
                        warn!(?err, "failed to decode gossip payload");
                        continue;
                    }
                };

                // Verify signature.
                let signer_id = payload.signer_id();
                let Ok(sig_bytes) = signed_msg.signature().try_into() else {
                    warn!("invalid signature length: {}", signed_msg.signature().len());
                    continue;
                };
                let sig = Signature::from_bytes(sig_bytes);

                if let Err(err) = signer_id.verify(signed_msg.payload_bytes(), &sig) {
                    warn!(?err, "invalid gossip signature");
                    continue;
                }

                match payload {
                    SpaceGossipMsg::Join(join) => {
                        handle_join_broadcast(state, join).await;
                    }
                    SpaceGossipMsg::ObjectClaim(claim) => {
                        handle_object_claim(state, &claim).await;
                    }
                    SpaceGossipMsg::ObjectRelease(release) => {
                        handle_object_release(state, &release).await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_join_broadcast(state: &NetworkThreadState, join: JoinBroadcast) {
    info!(endpoint = %join.endpoint.id, "got join broadcast");

    if state.outbound.get_async(&join.endpoint.id).await.is_some() {
        return;
    }

    info!(endpoint = %join.endpoint.id, "peer joined");

    let state = state.clone();
    let remote = join.endpoint;
    n0_future::task::spawn(async move {
        if let Err(err) = super::space::agent::outbound::handle_outbound(state, remote).await {
            error!(?err, "error handling space outbound");
        }
    });
}

async fn handle_object_claim(
    state: &NetworkThreadState,
    claim: &super::space::gossip::ObjectClaimBroadcast,
) {
    let record_hash = claim.object_id.record_hash();

    let Some(entry) = state.spaces.get_async(&record_hash).await else {
        warn!(?record_hash, "claim for unknown space");
        return;
    };

    let handle = entry.get();
    if handle
        .ownership
        .try_claim(claim.object_id, claim.claimer, claim.timestamp, claim.seq)
    {
        let _ = state
            .event_tx
            .try_send(NetworkEvent::ObjectOwnershipChanged {
                object_id: claim.object_id,
                owner: Some(claim.claimer),
            });
    }
}

async fn handle_object_release(
    state: &NetworkThreadState,
    release: &super::space::gossip::ObjectReleaseBroadcast,
) {
    let record_hash = release.object_id.record_hash();

    let Some(entry) = state.spaces.get_async(&record_hash).await else {
        return;
    };

    let handle = entry.get();
    if handle
        .ownership
        .release(release.object_id, release.releaser)
    {
        let _ = state
            .event_tx
            .try_send(NetworkEvent::ObjectOwnershipChanged {
                object_id: release.object_id,
                owner: None,
            });
    }
}
