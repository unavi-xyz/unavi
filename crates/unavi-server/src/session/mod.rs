use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use dwn::{Actor, Dwn, document_key::DocumentKey, stores::NativeDbStore};
use futures::StreamExt;
use tarpc::{
    server::{BaseChannel, Channel},
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use tokio::sync::{RwLock, watch};
use tokio_serde::formats::Bincode;
use tracing::{error, info};
use unavi_server_service::{
    ControlService, TrackingIFrame, TrackingPFrame, from_server::ControlMessage,
};
use wtransport::{endpoint::IncomingSession, stream::BiStream};
use xdid::{
    core::{did::Did, did_url::DidUrl},
    methods::{key::p256::P256KeyPair, web::reqwest::Url},
};

use crate::{DIRS, InternalMessage, session::control::ControlServer};

mod control;
mod init_space_host;
mod pull_transform;
mod streams;

pub const KEY_FRAGMENT: &str = "owner";

const TICKRATE: Duration = Duration::from_millis(50);
pub const DEFAULT_MAX_PLAYERS_PER_SPACE: usize = 32;

type PlayerId = u64;
type SpaceId = String;

#[derive(Clone)]
pub struct SessionSpawner {
    pub ctx: ServerContext,
    player_id_count: Arc<AtomicU64>,
    pub domain: String,
}

#[derive(Clone)]
pub struct ServerContext {
    pub actor: Actor,
    msg_tx: tokio::sync::mpsc::Sender<InternalMessage>,
    player_tickrates: Arc<RwLock<HashMap<PlayerId, HashMap<PlayerId, Duration>>>>,
    players: Arc<RwLock<HashMap<PlayerId, Player>>>,
    spaces: Arc<RwLock<HashMap<SpaceId, Space>>>,
}

impl ServerContext {
    /// Send player count update for a space.
    async fn update_space_player_count(&self, space_id: String, count: usize) {
        let _ = self
            .msg_tx
            .send(InternalMessage::SetPlayerCount {
                record_id: space_id,
                count,
            })
            .await;
    }
}

struct Player {
    spaces: HashSet<String>,
    iframe_tx: watch::Sender<TrackingIFrame>,
    pframe_tx: watch::Sender<TrackingPFrame>,
}

struct Space {
    players: HashSet<PlayerId>,
    max_players: usize,
    control_tx: tokio::sync::broadcast::Sender<ControlMessage>,
}

impl Default for Space {
    fn default() -> Self {
        let (control_tx, _) = tokio::sync::broadcast::channel(100);
        Self {
            players: HashSet::new(),
            max_players: DEFAULT_MAX_PLAYERS_PER_SPACE,
            control_tx,
        }
    }
}

#[derive(Clone)]
pub struct SpawnerOptions {
    pub did: Did,
    pub domain: String,
    pub in_memory: bool,
    pub msg_tx: tokio::sync::mpsc::Sender<InternalMessage>,
    pub remote: Url,
    pub vc: P256KeyPair,
}

impl SessionSpawner {
    pub async fn new(opts: SpawnerOptions) -> anyhow::Result<Self> {
        let store = if opts.in_memory {
            NativeDbStore::new_in_memory()?
        } else {
            let mut path = DIRS.data_local_dir().to_path_buf();
            path.push("data.db");
            NativeDbStore::new(path)?
        };
        let dwn = Dwn::from(store);

        let mut actor = Actor::new(opts.did.clone(), dwn);

        actor.remote = Some(opts.remote);

        let mut key: DocumentKey = opts.vc.into();
        key.url = DidUrl {
            did: opts.did,
            query: None,
            fragment: Some(KEY_FRAGMENT.to_string()),
            path_abempty: None,
        };

        let key = Arc::new(key);
        actor.sign_key = Some(key.clone());
        actor.auth_key = Some(key);

        actor.sync().await?;

        Ok(Self {
            ctx: ServerContext {
                actor,
                msg_tx: opts.msg_tx,
                player_tickrates: Default::default(),
                players: Default::default(),
                spaces: Default::default(),
            },
            domain: opts.domain,
            player_id_count: Default::default(),
        })
    }

    pub async fn handle_session(self, incoming: IncomingSession) -> anyhow::Result<()> {
        let req = incoming.await?;
        let con = req.accept().await?;

        let stream = con.accept_bi().await?;
        let bi_stream = BiStream::join(stream);
        let framed = Framed::new(bi_stream, LengthDelimitedCodec::default());
        let transport = tarpc::serde_transport::new(framed, Bincode::default());
        let channel = BaseChannel::with_defaults(transport).max_concurrent_requests(2);

        let player_id = self.player_id_count.fetch_add(1, Ordering::AcqRel);
        let server = ControlServer::new(self.ctx.clone(), player_id);

        tokio::spawn(channel.execute(server.serve()).for_each(|res| async move {
            tokio::spawn(res);
        }));

        {
            let (iframe_tx, _iframe_rx) = watch::channel(Default::default());
            let (pframe_tx, _pframe_rx) = watch::channel(Default::default());

            let mut players = self.ctx.players.write().await;
            players.insert(
                player_id,
                Player {
                    spaces: Default::default(),
                    iframe_tx,
                    pframe_tx,
                },
            );
        }

        tokio::spawn({
            let ctx = self.ctx.clone();
            let con = con.clone();
            async move {
                if let Err(e) = pull_transform::handle_pull_transforms(ctx, player_id, con).await {
                    error!("Error in pull transforms: {e:?}");
                }
            }
        });

        tokio::spawn({
            let ctx = self.ctx.clone();
            let con = con.clone();
            async move {
                if let Err(e) = streams::control::handle_control_stream(ctx, player_id, con).await {
                    error!("Error in control stream: {e:?}");
                }
            }
        });

        // Handle streams.
        tokio::spawn({
            let con = con.clone();
            let counts = streams::StreamCounts::default();
            let ctx = self.ctx.clone();

            async move {
                while let Ok(stream) = con.accept_uni().await {
                    let counts = counts.clone();
                    let ctx = ctx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = streams::handle_stream(ctx, counts, player_id, stream).await
                        {
                            error!("Error handling stream: {e:?}");
                        }
                    });
                }
            }
        });

        let err = con.closed().await;
        info!("Connection closed: {err}");
        self.cleanup_player(player_id).await;

        Ok(())
    }

    async fn cleanup_player(&self, player_id: PlayerId) {
        let spaces_to_leave: Vec<SpaceId> = {
            let mut players = self.ctx.players.write().await;
            if let Some(player) = players.remove(&player_id) {
                player.spaces.into_iter().collect()
            } else {
                Vec::new()
            }
        };

        let mut spaces = self.ctx.spaces.write().await;
        for space_id in spaces_to_leave {
            if let Some(space) = spaces.get_mut(&space_id) {
                space.players.remove(&player_id);

                // Broadcast PlayerLeft to all remaining players in this space.
                let _ = space
                    .control_tx
                    .send(ControlMessage::PlayerLeft { player_id });

                let count = space.players.len();
                if count == 0 {
                    spaces.remove(&space_id);
                }
                drop(spaces);

                self.ctx
                    .update_space_player_count(space_id.clone(), count)
                    .await;

                spaces = self.ctx.spaces.write().await;
            }
        }
        drop(spaces);

        let mut player_tickrates = self.ctx.player_tickrates.write().await;
        player_tickrates.remove(&player_id);
        for rates in player_tickrates.values_mut() {
            rates.remove(&player_id);
        }
    }
}
