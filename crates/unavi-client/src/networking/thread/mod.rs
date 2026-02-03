use std::{
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
use wds::{Blobs, DataStore, actor::Actor, identity::Identity};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

use crate::networking::thread::space::{
    DEFAULT_TICKRATE, IFrameMsg, PFrameDatagram, PlayerIFrame, PlayerPFrame,
};

mod join;
mod publish_beacon;
mod remote_wds;
pub mod space;

#[expect(unused)]
pub enum NetworkCommand {
    Join(Hash),
    Leave(Hash),
    PublishBeacon { id: Hash, ttl: Duration },
    PublishIFrame(PlayerIFrame),
    PublishPFrame(PlayerPFrame),
    Shutdown,
}

#[expect(unused)]
pub enum NetworkEvent {
    AddRemoteActor(Actor),
    SetLocalActor(Actor),
    SetLocalBlobs(Blobs),
    PlayerJoin {
        id: EndpointId,
        state: Arc<InboundState>,
    },
    PlayerLeave(EndpointId),
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

pub struct OutboundConn {
    task: AbortOnDropHandle<()>,
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
            tickrate: AtomicU8::new(DEFAULT_TICKRATE),
        }
    }
}

#[derive(Debug, Default)]
pub struct PoseState {
    pub iframe: Mutex<Option<IFrameMsg>>,
    pub pframe: Mutex<Option<PFrameDatagram>>,
}

#[derive(Clone)]
pub struct NetworkThreadState {
    pub endpoint: Endpoint,
    pub gossip: Gossip,
    pub local_actor: Actor,
    pub remote_actor: Option<Actor>,

    pub outbound: Arc<scc::HashMap<EndpointId, OutboundConn>>,
    pub _inbound: Arc<scc::HashMap<EndpointId, Arc<InboundState>>>,

    pub iframe_id: Arc<AtomicU16>,
    pub pose: Arc<PoseState>,
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

    let state = NetworkThreadState {
        endpoint,
        gossip,
        local_actor,
        remote_actor,
        outbound: Arc::default(),
        _inbound: inbound,
        // Initialize ID at a random value, to avoid leaking playtime information.
        iframe_id: Arc::new(AtomicU16::new(rand::random())),
        pose: Arc::default(),
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
            NetworkCommand::PublishIFrame(pose) => {
                let id = state.iframe_id.fetch_add(1, Ordering::Relaxed) + 1;
                let msg = IFrameMsg { id, pose };
                *state.pose.iframe.lock() = Some(msg);
            }
            NetworkCommand::PublishPFrame(pose) => {
                let iframe_id = state.iframe_id.load(Ordering::Relaxed);
                let msg = PFrameDatagram {
                    iframe_id,
                    seq: 0,
                    pose,
                };
                *state.pose.pframe.lock() = Some(msg);
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
