use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU16, Ordering},
    },
    time::Duration,
};

use bevy::{log::tracing::Instrument, prelude::*};
use blake3::Hash;
use iroh::{Endpoint, EndpointId};
use iroh_gossip::Gossip;
use n0_future::task::AbortOnDropHandle;
use parking_lot::Mutex;
use time::OffsetDateTime;

/// Returns the current time as milliseconds since Unix epoch.
#[expect(clippy::cast_possible_truncation)]
fn now_millis() -> u64 {
    (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000).cast_unsigned() as u64
}
use wds::{
    Blobs, DataStore,
    actor::Actor,
    identity::Identity,
    signed_bytes::{IrohSigner, Signable},
};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

use crate::networking::thread::space::{
    MAX_AGENT_TICKRATE, SpaceHandle,
    gossip::{ObjectClaimBroadcast, SpaceGossipMsg},
    msg::{AgentIFrameMsg, AgentPFrameDatagram, ObjectIFrameMsg, ObjectPFrameDatagram},
    types::{
        object_id::ObjectId,
        physics_state::{PhysicsIFrame, PhysicsPFrame},
        pose::{AgentIFrame, AgentPFrame},
    },
};

mod join;
mod publish_beacon;
mod remote_wds;
pub mod space;

pub enum NetworkCommand {
    Join(Hash),
    Leave(Hash),
    PublishBeacon { id: Hash, ttl: Duration },

    PublishAgentIFrame(AgentIFrame),
    PublishAgentPFrame(AgentPFrame),
    SetPeerTickrate { peer: EndpointId, tickrate: u8 },

    ClaimObject(ObjectId),
    PublishObjectIFrame(Vec<(ObjectId, PhysicsIFrame)>),
    PublishObjectPFrame(Vec<(ObjectId, PhysicsPFrame)>),
    UpdateGrabbedObjects(HashSet<ObjectId>),

    Shutdown,
}

#[expect(unused)]
pub enum NetworkEvent {
    AddRemoteActor(Actor),
    SetLocalActor(Actor),
    SetLocalBlobs(Blobs),
    SetLocalEndpoint(EndpointId),

    // Agent events.
    AgentJoin {
        id: EndpointId,
        state: Arc<InboundState>,
    },
    AgentLeave(EndpointId),

    // Object events.
    ObjectOwnershipChanged {
        object_id: ObjectId,
        owner: Option<EndpointId>,
    },
    ObjectPoseUpdate {
        source: EndpointId,
        objects: Vec<(ObjectId, PhysicsIFrame)>,
    },

    /// Object grab state changed from remote owner.
    ObjectGrabChanged {
        object_id: ObjectId,
        grabbed: bool,
    },
}

#[derive(Resource)]
pub struct NetworkingThread {
    pub command_tx: tokio::sync::mpsc::Sender<NetworkCommand>,
    pub event_rx: tokio::sync::mpsc::Receiver<NetworkEvent>,
}

pub struct NetworkingThreadOpts {
    pub wds_in_memory: bool,
}

impl NetworkingThread {
    pub fn spawn(opts: NetworkingThreadOpts) -> Self {
        let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(16);
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(32);

        unavi_wasm_compat::spawn_thread(async move {
            while let Err(err) = thread_loop(&opts, &mut command_rx, &event_tx).await {
                error!(?err, "Networking thread error");
                n0_future::time::sleep(Duration::from_secs(5)).await;
            }
        });

        Self {
            command_tx,
            event_rx,
        }
    }
}

/// Connection state for outbound streaming (agent and object).
pub struct OutboundConn {
    pub task: AbortOnDropHandle<()>,
    pub tickrate: Arc<AtomicU8>,
    /// The underlying connection for opening object streams.
    pub connection: iroh::endpoint::Connection,
}

#[derive(Debug)]
pub struct InboundState {
    pub pose: PoseState,
    pub tickrate: AtomicU8,
}

impl Default for InboundState {
    fn default() -> Self {
        Self {
            pose: PoseState::default(),
            tickrate: AtomicU8::new(MAX_AGENT_TICKRATE),
        }
    }
}

#[derive(Debug, Default)]
pub struct PoseState {
    pub iframe: Mutex<Option<AgentIFrameMsg>>,
    pub pframe: Mutex<Option<AgentPFrameDatagram>>,
}

/// Per-object pose state for outbound streaming.
#[derive(Debug, Default)]
pub struct ObjectPoseState {
    /// Map of object ID to its current I-frame.
    pub iframes: scc::HashMap<ObjectId, ObjectIFrameMsg>,
    /// Map of object ID to its current P-frame.
    pub pframes: scc::HashMap<ObjectId, ObjectPFrameDatagram>,
}

#[derive(Clone)]
pub struct NetworkThreadState {
    pub endpoint: Endpoint,
    pub gossip: Gossip,
    pub local_actor: Actor,
    pub remote_actor: Option<Actor>,
    pub event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,

    /// Outbound connections
    pub outbound: Arc<scc::HashMap<EndpointId, OutboundConn>>,
    pub _inbound: Arc<scc::HashMap<EndpointId, Arc<InboundState>>>,

    /// Per-space gossip senders and ownership tables.
    pub spaces: Arc<scc::HashMap<Hash, SpaceHandle>>,

    pub iframe_id: Arc<AtomicU16>,
    pub pose: Arc<PoseState>,

    pub object_iframe_id: Arc<AtomicU16>,
    pub object_pose: Arc<ObjectPoseState>,

    /// Watch channel for broadcasting grabbed objects to outbound streams.
    pub grabbed_objects_rx: tokio::sync::watch::Receiver<HashSet<ObjectId>>,
}

#[expect(clippy::too_many_lines)]
async fn thread_loop(
    opts: &NetworkingThreadOpts,
    command_rx: &mut tokio::sync::mpsc::Receiver<NetworkCommand>,
    event_tx: &tokio::sync::mpsc::Sender<NetworkEvent>,
) -> anyhow::Result<()> {
    let endpoint = Endpoint::builder();

    #[cfg(feature = "mdns")]
    let endpoint = endpoint.discovery(iroh::discovery::mdns::MdnsDiscovery::builder());

    let endpoint = endpoint.bind().await?;
    info!("Local endpoint: {}", endpoint.id());

    event_tx
        .send(NetworkEvent::SetLocalEndpoint(endpoint.id()))
        .await?;

    let gossip = Gossip::builder().spawn(endpoint.clone());
    let inbound = Arc::new(scc::HashMap::default());

    let store = {
        let mut builder = DataStore::builder(endpoint.clone());

        #[cfg(not(target_family = "wasm"))]
        if !opts.wds_in_memory {
            let path = crate::assets::DIRS.data_local_dir().join("wds");
            builder = builder.storage_path(path);
        }

        let space_protocol = space::SpaceProtocol {
            event_tx: event_tx.clone(),
            inbound: Arc::clone(&inbound),
        };

        builder
            .accept(iroh_gossip::ALPN, gossip.clone())
            .accept(space::ALPN, space_protocol)
            .gc_timer(Duration::from_mins(15))
            .build()
            .await?
    };

    let blobs = store.blobs().blobs();
    event_tx
        .send(NetworkEvent::SetLocalBlobs(blobs.clone()))
        .await?;

    // TODO: save / load keypair from disk
    let signing_key = P256KeyPair::generate();
    let did = signing_key.public().to_did();
    info!("Local identity: {did}");

    let identity = Arc::new(Identity::new(did, signing_key));
    store.set_user_identity(Arc::clone(&identity));

    let local_actor = store.local_actor(Arc::clone(&identity));

    let remote_host = remote_wds::fetch_remote_host().await?;
    let remote_actor = remote_host.map(|h| store.remote_actor(identity, h));

    event_tx
        .send(NetworkEvent::SetLocalActor(local_actor.clone()))
        .await?;

    if let Some(ref remote_actor) = remote_actor {
        event_tx
            .send(NetworkEvent::AddRemoteActor(remote_actor.clone()))
            .await?;
    }

    // Initialize IDs at random values to avoid leaking playtime.
    let (grabbed_objects_tx, grabbed_objects_rx) = tokio::sync::watch::channel(HashSet::default());
    let state = NetworkThreadState {
        endpoint,
        gossip,
        local_actor,
        remote_actor,
        event_tx: event_tx.clone(),
        outbound: Arc::default(),
        _inbound: inbound,
        spaces: Arc::default(),
        iframe_id: Arc::new(AtomicU16::new(rand::random())),
        pose: Arc::default(),
        object_iframe_id: Arc::new(AtomicU16::new(rand::random())),
        object_pose: Arc::default(),
        grabbed_objects_rx,
    };

    while let Some(cmd) = command_rx.recv().await {
        match cmd {
            NetworkCommand::Join(id) => {
                let state = state.clone();
                let span = info_span!("", space = %id);

                n0_future::task::spawn(
                    async move {
                        if let Err(err) = join::handle_join(state, id).await {
                            error!(?err, "error joining space");
                        }
                    }
                    .instrument(span),
                );
            }
            NetworkCommand::Leave(_id) => {
                todo!()
            }
            NetworkCommand::PublishBeacon { id, ttl } => {
                let state = state.clone();

                n0_future::task::spawn(async move {
                    if let Err(err) = publish_beacon::publish_beacon(state, id, ttl).await {
                        error!(?err, "error publishing beacon");
                    }
                });
            }
            NetworkCommand::PublishAgentIFrame(pose) => {
                let id = state.iframe_id.fetch_add(1, Ordering::Relaxed) + 1;
                let msg = AgentIFrameMsg { id, pose };
                *state.pose.iframe.lock() = Some(msg);
            }
            NetworkCommand::PublishAgentPFrame(pose) => {
                let iframe_id = state.iframe_id.load(Ordering::Relaxed);
                let msg = AgentPFrameDatagram {
                    iframe_id,
                    seq: 0,
                    pose,
                };
                *state.pose.pframe.lock() = Some(msg);
            }
            NetworkCommand::SetPeerTickrate { peer, tickrate } => {
                state
                    .outbound
                    .read_async(&peer, |_, conn| {
                        conn.tickrate.store(tickrate, Ordering::Relaxed);
                    })
                    .await;
            }
            NetworkCommand::ClaimObject(object_id) => {
                let record_hash = object_id.record;
                let endpoint_id = state.endpoint.id();
                let now = now_millis();

                if let Some(entry) = state.spaces.get_async(&record_hash).await {
                    let handle = entry.get();
                    if handle.ownership.try_claim(object_id, endpoint_id, now, 0) {
                        let claim = SpaceGossipMsg::ObjectClaim(ObjectClaimBroadcast {
                            object_id,
                            claimer: endpoint_id,
                            timestamp: now,
                            seq: 0,
                        });
                        if let Err(err) = broadcast_gossip(&state, &claim, &handle.gossip_tx).await
                        {
                            error!(?err, "failed to broadcast claim");
                        }
                        let _ = event_tx.try_send(NetworkEvent::ObjectOwnershipChanged {
                            object_id,
                            owner: Some(endpoint_id),
                        });
                    }
                } else {
                    warn!(?record_hash, "claim for unknown space");
                }
            }
            NetworkCommand::PublishObjectIFrame(objects) => {
                let id = state.object_iframe_id.fetch_add(1, Ordering::Relaxed) + 1;
                let now = now_millis();
                for (object_id, physics_state) in objects {
                    let msg = ObjectIFrameMsg {
                        id,
                        timestamp: now,
                        state: physics_state,
                    };
                    let _ = state.object_pose.iframes.upsert_sync(object_id, msg);
                }
            }
            NetworkCommand::PublishObjectPFrame(objects) => {
                let iframe_id = state.object_iframe_id.load(Ordering::Relaxed);
                let now = now_millis();
                for (object_id, physics_state) in objects {
                    let msg = ObjectPFrameDatagram {
                        iframe_id,
                        seq: 0,
                        timestamp: now,
                        object_id,
                        state: physics_state,
                    };
                    let _ = state.object_pose.pframes.upsert_sync(object_id, msg);
                }
            }
            NetworkCommand::UpdateGrabbedObjects(objects) => {
                let _ = grabbed_objects_tx.send(objects);
            }

            NetworkCommand::Shutdown => {
                if let Err(err) = store.shutdown().await {
                    error!(?err, "error shutting down data store");
                }

                break;
            }
        }
    }

    info!("Graceful exit");

    Ok(())
}

/// Sign and broadcast a gossip message.
async fn broadcast_gossip(
    state: &NetworkThreadState,
    msg: &SpaceGossipMsg,
    gossip_tx: &iroh_gossip::api::GossipSender,
) -> anyhow::Result<()> {
    let signed = msg.sign(&IrohSigner(state.endpoint.secret_key()))?;
    let bytes = postcard::to_stdvec(&signed)?;
    gossip_tx.broadcast(bytes.into()).await?;
    Ok(())
}
