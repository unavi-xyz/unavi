use std::{collections::BTreeSet, sync::Arc, time::Duration};

use anyhow::bail;
use bevy::{
    log::{error, info, warn},
    tasks::futures_lite::StreamExt,
};
use blake3::Hash;
use iroh::{EndpointAddr, EndpointId, PublicKey, Signature};
use iroh_gossip::{
    TopicId,
    api::{Event, GossipReceiver, GossipSender, JoinOptions},
};
use time::OffsetDateTime;
use wds::signed_bytes::{IrohSigner, Signable, SignedBytes};
use wired_records::BeaconRecord;
use wired_schemas::SCHEMA_BEACON;

use crate::networking::thread::{
    NetworkEvent, NetworkThreadState,
    space::{
        SpaceHandle,
        gossip::{SpaceGossip, SpaceGossipMsg},
        ownership::ObjectOwnership,
    },
};

pub async fn handle_join(state: NetworkThreadState, space: Hash) -> anyhow::Result<()> {
    if state.spaces.contains_async(&space).await {
        return Ok(());
    }

    // Bootstrap from beacons.
    let bootstrap = find_bootstrap_peers(&state, space).await?;
    info!(bootstrap = bootstrap.len(), "joining gossip topic");

    // Join gossip topic.
    let topic_id = TopicId::from_bytes(*space.as_bytes());
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

    // Register space handle with cancel signal.
    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
    let ownership = Arc::new(ObjectOwnership::new());
    let handle = SpaceHandle {
        gossip_tx: tx.clone(),
        ownership,
        cancel_tx,
    };
    if state.spaces.insert_async(space, handle).await.is_err() {
        bail!("space already joined?")
    }

    // Handle gossip events.
    let state = Arc::new(state);

    let gossip_task = tokio::spawn({
        let state = Arc::clone(&state);
        async move {
            loop {
                match handle_gossip_inbound(&state, &tx, &mut rx, space).await {
                    Ok(()) => {
                        break;
                    }
                    Err(err) => {
                        error!(?err, "error handling inbound gossip");
                        n0_future::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    });

    tokio::select! {
        res = cancel_rx => {
            if let Err(err) = res {
                error!(?err);
            }
        }
        res = gossip_task => {
            if let Err(err) = res {
                error!(?err);
            }
        }
    }

    // Clean up space handle on exit.
    let _ = state.spaces.remove_async(&space).await;

    Ok(())
}

async fn find_bootstrap_peers(
    state: &NetworkThreadState,
    space: Hash,
) -> anyhow::Result<BTreeSet<PublicKey>> {
    let mut bootstrap = BTreeSet::new();

    // Search for beacons, ideally from a remote actor but fallback to local.
    let target_actor = state.remote_actor.as_ref().unwrap_or(&state.local_actor);
    let found = target_actor
        .query()
        .schema(SCHEMA_BEACON.hash)
        .send()
        .await?;

    let now = OffsetDateTime::now_utc().unix_timestamp();

    for id in found {
        let mut builder = state.local_actor.read(id);

        if let Some(remote) = &state.remote_actor {
            builder = builder.sync_from(remote.host().clone());
        }

        match builder.send().await {
            Ok(doc) => {
                let Ok(beacon) = BeaconRecord::load(&doc) else {
                    continue;
                };
                if now >= beacon.expires {
                    continue;
                }
                if beacon.space.0 != space {
                    continue;
                }
                let Ok(endpoint) = EndpointId::from_bytes(&beacon.endpoint.0) else {
                    continue;
                };
                if endpoint == state.endpoint.id() {
                    continue;
                }
                bootstrap.insert(endpoint);
            }
            Err(err) => {
                warn!(?err, "failed to sync beacon");
            }
        }
    }

    Ok(bootstrap)
}

async fn handle_gossip_inbound(
    state: &NetworkThreadState,
    tx: &GossipSender,
    rx: &mut GossipReceiver,
    space_id: Hash,
) -> anyhow::Result<()> {
    while let Some(event) = rx.next().await {
        match event? {
            Event::NeighborUp(n) => {
                info!("+neighbor: {n}");

                // Broadcast join whenever we gain a new neighbor.
                let msg = SpaceGossip {
                    sender: state.endpoint.id(),
                    msg: SpaceGossipMsg::Join(state.endpoint.addr()),
                };
                let signed = msg.sign(&IrohSigner(state.endpoint.secret_key()))?;
                let bytes = postcard::to_stdvec(&signed)?;
                tx.broadcast(bytes.into()).await?;
            }
            Event::NeighborDown(n) => {
                info!("-neighbor: {n}");
                state.event_tx.try_send(NetworkEvent::PeerLeft(n))?;
            }
            Event::Lagged => bail!("lagged"),
            Event::Received(msg) => {
                let signed_msg =
                    match postcard::from_bytes::<SignedBytes<SpaceGossip>>(&msg.content) {
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
                let Ok(sig_bytes) = signed_msg.signature().try_into() else {
                    warn!("invalid signature length: {}", signed_msg.signature().len());
                    continue;
                };
                let sig = Signature::from_bytes(sig_bytes);

                if let Err(err) = payload.sender.verify(signed_msg.payload_bytes(), &sig) {
                    warn!(?err, "invalid gossip signature");
                    continue;
                }

                match payload.msg {
                    SpaceGossipMsg::Join(addr) => {
                        if addr.id != payload.sender {
                            warn!("join address does not match sender");
                            continue;
                        }

                        handle_join_broadcast(state, payload.sender, addr, space_id).await;
                    }
                    SpaceGossipMsg::StateDelta(delta) => {
                        let _ = state.event_tx.try_send(NetworkEvent::PeerStateDelta {
                            peer: delta.sender,
                            delta,
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_join_broadcast(
    state: &NetworkThreadState,
    peer: EndpointId,
    addr: EndpointAddr,
    space: Hash,
) {
    // If already connected to peer, ignore.
    // TODO track peer's joined spaces?
    let _ = state
        .event_tx
        .try_send(NetworkEvent::PeerJoinedSpace { peer, space });
    if state.outbound.get_async(&peer).await.is_some() {
        return;
    }

    info!(%peer, "peer joined, opening outbound connection");

    let state = state.clone();
    n0_future::task::spawn(async move {
        if let Err(err) = super::space::outbound::connect_to_peer(state, addr).await {
            error!(?err, "error handling outbound connection");
        }
    });
}
