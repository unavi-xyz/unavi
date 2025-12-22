use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use blake3::Hash;
use iroh::Endpoint;
use iroh_gossip::Gossip;
use iroh_tickets::endpoint::EndpointTicket;
use wds::actor::Actor;
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

use crate::DIRS;

mod command;
mod discovery;

pub enum NetworkCommand {
    Join(Hash),
    Leave(Hash),
    Shutdown,
}

pub enum NetworkEvent {
    SetActor(Arc<Actor>),
    Connected { id: Hash, space: Entity },
    ConnectionClosed { id: Hash, message: String },
}

#[derive(Resource)]
pub struct NetworkingThread {
    pub command_tx: flume::Sender<NetworkCommand>,
    pub event_rx: flume::Receiver<NetworkEvent>,
}

const CHANNEL_LEN: usize = 32;

impl NetworkingThread {
    pub fn spawn(peers: Vec<EndpointTicket>) -> Self {
        let (command_tx, command_rx) = flume::bounded(CHANNEL_LEN);
        let (event_tx, event_rx) = flume::bounded(CHANNEL_LEN);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("networking")
                .build()
                .expect("build tokio runtime");

            rt.block_on(async move {
                while let Err(e) = thread_loop(&command_rx, &event_tx, &peers).await {
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

#[derive(Clone)]
struct ThreadState {
    // actor: Arc<Actor>,
    endpoint: Endpoint,
    gossip: Gossip,
    // connections: scc::HashMap<RecordId>
}

async fn thread_loop(
    _command_rx: &flume::Receiver<NetworkCommand>,
    _event_tx: &flume::Sender<NetworkEvent>,
    peers: &[EndpointTicket],
) -> anyhow::Result<()> {
    // TODO: save / load keypair from disk
    let keypair = P256KeyPair::generate();

    let did = keypair.public().to_did();
    info!("Local identity: {did}");

    let _data_dir = {
        let mut path = DIRS.data_local_dir().to_path_buf();
        path.push("wds");
        path
    };

    // This is a dumb pattern but whatever.
    // let (gtx, grx) = tokio::sync::oneshot::channel();
    //
    // let store = DataStoreBuilder {
    //     data_dir,
    //     ephemeral: true,
    //     with_router: Some(Box::new(move |endpoint, r| {
    //         let gossip = Gossip::builder().spawn(endpoint.clone());
    //         gtx.send(gossip.clone())
    //             .expect("send gossip out of closure");
    //         r.accept(iroh_gossip::ALPN, gossip)
    //     })),
    // }
    // .build()
    // .await?;
    //
    // let gossip = grx.await?;
    //
    // let view = ValidatedView::new(store.view_for_user(did.clone()))?;
    // let actor = Arc::new(Actor::new(did, keypair, view));

    // event_tx
    //     .send_async(NetworkEvent::SetActor(Arc::clone(&actor)))
    //     .await?;
    //
    // for addr in store.endpoint().addr().ip_addrs() {
    //     info!("Endpoint listening on port {}", addr.port());
    // }
    //
    // let ticket = EndpointTicket::new(store.endpoint().addr());
    // info!("Share this ticket to connect to others: {ticket}");
    //
    // let state = ThreadState {
    //     actor,
    //     endpoint: store.endpoint().clone(),
    //     gossip,
    // };

    // TODO: Bootstrap from UNAVI hosted WDS endpoint
    let bootstrap = peers
        .iter()
        .map(|p| p.endpoint_addr().id)
        .collect::<Vec<_>>();
    info!("Bootstrap: {bootstrap:#?}");

    // let (beacon_tx, mut beacon_rx) = tokio::sync::mpsc::channel(8);

    // tokio::spawn({
    //     let bootstrap = bootstrap.clone();
    //     let state = state.clone();
    //
    //     async move {
    //         while let Err(e) =
    //             discovery::handle_space_discovery(bootstrap.clone(), state.clone(), &mut beacon_rx)
    //                 .await
    //         {
    //             error!("Error handling space discovery: {e:?}");
    //             tokio::time::sleep(Duration::from_secs(5)).await;
    //         }
    //     }
    // });
    //
    // loop {
    //     match command_rx.recv_async().await? {
    //         NetworkCommand::Join(id) => {
    //             let state = state.clone();
    //             let bootstrap = bootstrap.clone();
    //
    //             beacon_tx.send(id).await?;
    //
    //             tokio::spawn(async move {
    //                 if let Err(e) = command::join::handle_join(state, id, bootstrap).await {
    //                     error!("Error joining {id}: {e:?}");
    //                 }
    //             });
    //         }
    //         NetworkCommand::Leave(_id) => {
    //             // TODO
    //         }
    //         NetworkCommand::Shutdown => {
    //             if let Err(e) = store.router().shutdown().await {
    //                 error!("Error shutting down router: {e}");
    //             }
    //
    //             break;
    //         }
    //     }
    // }

    info!("Graceful exit");

    Ok(())
}
