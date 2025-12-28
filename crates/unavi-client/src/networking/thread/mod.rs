use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use blake3::Hash;
use wds::{DataStore, actor::Actor};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

use crate::DIRS;

mod join;

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
    actor: Arc<Actor>,
}

async fn thread_loop(
    command_rx: &flume::Receiver<NetworkCommand>,
    event_tx: &flume::Sender<NetworkEvent>,
) -> anyhow::Result<()> {
    let store = {
        let mut path = DIRS.data_local_dir().to_path_buf();
        path.push("wds");
        DataStore::new(&path).await?
    };

    // TODO: save / load keypair from disk
    let signing_key = P256KeyPair::generate();
    let did = signing_key.public().to_did();
    info!("Local identity: {did}");

    let actor = Arc::new(store.actor(did, signing_key));

    event_tx
        .send_async(NetworkEvent::SetActor(Arc::clone(&actor)))
        .await?;

    let state = NetworkThreadState { actor };

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
