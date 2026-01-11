use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use bevy::{log::tracing::Instrument, prelude::*};
use blake3::Hash;
use iroh::{Endpoint, EndpointId};
use iroh_gossip::Gossip;
use parking_lot::Mutex;
use tokio::task::JoinHandle;
use wds::{DataStore, actor::Actor, identity::Identity};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

use crate::{
    DIRS,
    networking::{
        WdsActors,
        thread::space::{IFrameMsg, PFrameDatagram, PlayerIFrame, PlayerPFrame},
    },
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
    SetActors(WdsActors),
    PlayerJoin {
        id: EndpointId,
        state: Arc<InboundState>,
    },
    PlayerLeave(EndpointId),
}

#[derive(Resource)]
pub struct NetworkingThread {
    pub command_tx: flume::Sender<NetworkCommand>,
    pub event_rx: flume::Receiver<NetworkEvent>,
}

const CHANNEL_LEN: usize = 32;

pub struct NetworkingThreadOpts {
    pub wds_in_memory: bool,
}

impl NetworkingThread {
    pub fn spawn(opts: NetworkingThreadOpts) -> Self {
        let (command_tx, command_rx) = flume::bounded(CHANNEL_LEN);
        let (event_tx, event_rx) = flume::bounded(CHANNEL_LEN);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("networking")
                .build()
                .expect("build tokio runtime");

            rt.block_on(async move {
                while let Err(e) = thread_loop(&opts, &command_rx, &event_tx).await {
                    error!("Networking thread error: {e:?}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });
        });

        Self {
            command_tx,
            event_rx,
        }
    }
}

pub struct OutboundConn {
    task: JoinHandle<()>,
}

#[derive(Debug, Default)]
pub struct InboundState {
    pub pose: PoseState,
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
    pub inbound: Arc<scc::HashMap<EndpointId, Arc<InboundState>>>,

    pub iframe_id: Arc<AtomicU32>,
    pub pose: Arc<PoseState>,
}

#[allow(clippy::too_many_lines)]
async fn thread_loop(
    opts: &NetworkingThreadOpts,
    command_rx: &flume::Receiver<NetworkCommand>,
    event_tx: &flume::Sender<NetworkEvent>,
) -> anyhow::Result<()> {
    let endpoint = Endpoint::builder().bind().await?;
    info!("Local endpoint: {}", endpoint.id());

    let gossip = Gossip::builder().spawn(endpoint.clone());
    let inbound = Arc::new(scc::HashMap::default());

    let store = {
        let mut builder = DataStore::builder(endpoint.clone());

        if !opts.wds_in_memory {
            let path = DIRS.data_local_dir().join("wds");
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
        .send_async(NetworkEvent::SetActors(WdsActors {
            local: local_actor.clone(),
            remote: remote_actor.clone(),
        }))
        .await?;

    let state = NetworkThreadState {
        endpoint,
        gossip,
        local_actor,
        remote_actor,
        outbound: Arc::default(),
        inbound,
        // Initialize ID at a random value, to avoid leaking information about playtime.
        iframe_id: Arc::new(AtomicU32::new(rand::random())),
        pose: Arc::default(),
    };

    loop {
        match command_rx.recv_async().await? {
            NetworkCommand::Join(id) => {
                let state = state.clone();
                let span = info_span!("", space = %id);

                tokio::spawn(
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

                tokio::spawn(async move {
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
                let msg = PFrameDatagram { iframe_id, pose };
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
