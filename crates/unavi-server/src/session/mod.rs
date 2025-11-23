use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use dwn::{Actor, Dwn, document_key::DocumentKey, stores::NativeDbStore};
use futures::StreamExt;
use scc::HashMap as SccHashMap;
use tarpc::{
    server::{BaseChannel, Channel},
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use tokio::sync::watch;
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

const TICKRATE: Duration = Duration::from_millis(70);
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
    msg_tx: flume::Sender<InternalMessage>,
    player_tickrates: Arc<SccHashMap<PlayerId, Arc<SccHashMap<PlayerId, Duration>>>>,
    players: Arc<SccHashMap<PlayerId, Player>>,
    spaces: Arc<SccHashMap<SpaceId, Space>>,
}

impl ServerContext {
    /// Send player count update for a space.
    async fn update_space_player_count(&self, space_id: String, count: usize) {
        let _ = self
            .msg_tx
            .send_async(InternalMessage::SetPlayerCount {
                record_id: space_id,
                count,
            })
            .await;
    }
}

#[derive(Clone)]
struct Player {
    spaces: HashSet<String>,
    iframe_tx: watch::Sender<TrackingIFrame>,
    pframe_tx: watch::Sender<TrackingPFrame>,
}

#[derive(Clone)]
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
    pub msg_tx: flume::Sender<InternalMessage>,
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
                player_tickrates: Arc::new(SccHashMap::new()),
                players: Arc::new(SccHashMap::new()),
                spaces: Arc::new(SccHashMap::new()),
            },
            domain: opts.domain,
            player_id_count: Arc::default(),
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

        let (iframe_tx, _iframe_rx) = watch::channel(TrackingIFrame::default());
        let (pframe_tx, _pframe_rx) = watch::channel(TrackingPFrame::default());

        let _ = self
            .ctx
            .players
            .insert_async(
                player_id,
                Player {
                    spaces: HashSet::default(),
                    iframe_tx,
                    pframe_tx,
                },
            )
            .await;

        let tasks = [
            tokio::spawn(channel.execute(server.serve()).for_each(|res| async move {
                tokio::spawn(res);
            }))
            .abort_handle(),
            tokio::spawn({
                let ctx = self.ctx.clone();
                let con = con.clone();
                async move {
                    if let Err(e) =
                        pull_transform::handle_pull_transforms(ctx, player_id, con).await
                    {
                        error!("Error in pull transforms: {e:?}");
                    }
                }
            })
            .abort_handle(),
            tokio::spawn({
                let ctx = self.ctx.clone();
                let con = con.clone();
                async move {
                    if let Err(e) =
                        streams::control::handle_control_stream(ctx, player_id, con).await
                    {
                        error!("Error in control stream: {e:?}");
                    }
                }
            })
            .abort_handle(),
            tokio::spawn({
                let con = con.clone();
                let counts = streams::StreamCounts::default();
                let ctx = self.ctx.clone();

                async move {
                    while let Ok(stream) = con.accept_uni().await {
                        let counts = counts.clone();
                        let ctx = ctx.clone();
                        tokio::spawn(async move {
                            if let Err(e) =
                                streams::recv_stream(ctx, counts, player_id, stream).await
                            {
                                error!("Error handling stream: {e:?}");
                            }
                        });
                    }
                }
            })
            .abort_handle(),
        ];

        let err = con.closed().await;
        info!("Connection closed: {err}");
        self.cleanup_player(player_id).await;

        for t in tasks {
            t.abort();
        }

        Ok(())
    }

    async fn cleanup_player(&self, player_id: PlayerId) {
        let spaces_to_leave: Vec<SpaceId> = {
            if let Some((_, player)) = self.ctx.players.remove_async(&player_id).await {
                player.spaces.into_iter().collect()
            } else {
                Vec::new()
            }
        };

        for space_id in spaces_to_leave {
            let control_tx = self
                .ctx
                .spaces
                .read_async(&space_id, |_, space| space.control_tx.clone())
                .await;

            let updated = self
                .ctx
                .spaces
                .update_async(&space_id, |_, space| {
                    space.players.remove(&player_id);
                    let count = space.players.len();
                    (count == 0, count)
                })
                .await;

            if let Some(tx) = control_tx {
                let _ = tx.send(ControlMessage::PlayerLeft { player_id });
            }

            if let Some((should_remove, count)) = updated {
                if should_remove {
                    let _ = self.ctx.spaces.remove_async(&space_id).await;
                }

                self.ctx
                    .update_space_player_count(space_id.clone(), count)
                    .await;
            }
        }

        let _ = self.ctx.player_tickrates.remove_async(&player_id).await;

        // Remove this player from all other players' tickrate maps.
        // Collect keys first using any() which allows us to iterate.
        let mut keys = Vec::new();
        let _ = self.ctx.player_tickrates.iter_sync(|k, _| {
            keys.push(*k);
            false // Never early-exit
        });

        for key in keys {
            let rates = self
                .ctx
                .player_tickrates
                .read_async(&key, |_, rates| rates.clone())
                .await;

            if let Some(rates) = rates {
                let _ = rates.remove_async(&player_id).await;
            }
        }
    }
}
