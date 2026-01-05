use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use blake3::Hash;
use iroh::Endpoint;
use wds::{DataStore, Identity, actor::Actor};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

use crate::{DIRS, networking::WdsActors};

mod join;
mod remote_wds;

#[expect(unused)]
pub enum NetworkCommand {
    Join(Hash),
    Leave(Hash),
    Shutdown,
}

#[expect(unused)]
pub enum NetworkEvent {
    SetActors(WdsActors),
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
                while let Err(e) = thread_loop(&command_rx, &event_tx).await {
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
struct NetworkThreadState {
    local_actor: Actor,
    remote_actor: Option<Actor>,
}

async fn thread_loop(
    command_rx: &flume::Receiver<NetworkCommand>,
    event_tx: &flume::Sender<NetworkEvent>,
) -> anyhow::Result<()> {
    let endpoint = Endpoint::builder().bind().await?;

    let store = {
        let path = DIRS.data_local_dir().join("wds");
        DataStore::new(&path, endpoint).await?
    };

    // TODO: save / load keypair from disk
    let signing_key = P256KeyPair::generate();
    let did = signing_key.public().to_did();
    info!("Local identity: {did}");

    let identity = Arc::new(Identity::new(did, signing_key));
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
        local_actor,
        remote_actor,
    };

    loop {
        match command_rx.recv_async().await? {
            NetworkCommand::Join(id) => {
                let state = state.clone();

                tokio::spawn(async move {
                    if let Err(e) = join::handle_join(state, id).await {
                        error!("Error joining {id}: {e:?}");
                    }
                });
            }
            NetworkCommand::Leave(_id) => {
                todo!()
            }
            NetworkCommand::Shutdown => {
                if let Err(e) = store.shutdown().await {
                    error!("Error shutting down data store: {e}");
                }

                break;
            }
        }
    }

    info!("Graceful exit");

    Ok(())
}
