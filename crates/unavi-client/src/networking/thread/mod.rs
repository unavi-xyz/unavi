use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use iroh::Endpoint;
use iroh_gossip::Gossip;
use iroh_tickets::endpoint::EndpointTicket;
use wired_data_store::{Actor, DataStore, RecordId, ValidatedView};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

use crate::DIRS;

mod command;

pub enum NetworkCommand {
    Join(RecordId),
    Leave(RecordId),
    Shutdown,
}

pub enum NetworkEvent {
    SetActor(Arc<Actor>),
    Connected { id: RecordId, space: Entity },
    ConnectionClosed { id: RecordId, message: String },
}

#[derive(Resource)]
pub struct NetworkingThread {
    pub command_tx: flume::Sender<NetworkCommand>,
    pub event_rx: flume::Receiver<NetworkEvent>,
}

const CHANNEL_LEN: usize = 32;

impl NetworkingThread {
    pub fn spawn() -> Self {
        let (command_tx, command_rx) = flume::bounded(CHANNEL_LEN);
        let (event_tx, event_rx) = flume::bounded(CHANNEL_LEN);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("networking")
                .build()
                .expect("build tokio runtime");

            rt.block_on(async move {
                loop {
                    if let Err(e) = thread_loop(&command_rx, &event_tx).await {
                        error!("Networking thread error: {e:?}");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            });
        });

        Self {
            command_tx,
            event_rx,
        }
    }
}

#[derive(Clone)]
struct ThreadState {
    actor: Arc<Actor>,
    endpoint: Endpoint,
    gossip: Gossip,
    // connections: scc::HashMap<RecordId>
}

async fn thread_loop(
    command_rx: &flume::Receiver<NetworkCommand>,
    event_tx: &flume::Sender<NetworkEvent>,
) -> anyhow::Result<()> {
    // TODO: save / load keypair from disk
    let keypair = P256KeyPair::generate();

    let did = keypair.public().to_did();
    info!("Local identity: {did}");

    let data_dir = {
        let mut path = DIRS.data_local_dir().to_path_buf();
        path.push("wds");
        path
    };

    // This is a dumb pattern but whatever.
    let (gtx, grx) = tokio::sync::oneshot::channel();

    let store = DataStore::new_with_router(data_dir, move |endpoint, r| {
        let gossip = Gossip::builder().spawn(endpoint.clone());
        gtx.send(gossip.clone())
            .expect("send gossip out of closure");
        r.accept(iroh_gossip::ALPN, gossip)
    })
    .await?;

    let gossip = grx.await?;

    let view = ValidatedView::new(store.view_for_user(did.clone()))?;
    let actor = Arc::new(Actor::new(did, keypair, view));

    event_tx
        .send_async(NetworkEvent::SetActor(Arc::clone(&actor)))
        .await?;

    let ticket = EndpointTicket::new(store.endpoint().addr());
    info!("Share this ticket to connect to others: {ticket}");

    let state = ThreadState {
        actor,
        endpoint: store.endpoint().clone(),
        gossip,
    };

    loop {
        match command_rx.recv_async().await? {
            NetworkCommand::Join(id) => {
                let state = state.clone();
                tokio::spawn(async move { command::join::handle_join(state, id).await });
            }
            NetworkCommand::Leave(_id) => {
                // TODO
            }
            NetworkCommand::Shutdown => {
                if let Err(e) = store.router().shutdown().await {
                    error!("Error shutting down router: {e}");
                }

                break;
            }
        }
    }

    Ok(())
}
