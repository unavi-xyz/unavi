use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

use dwn::{Actor, Dwn, document_key::DocumentKey, stores::NativeDbStore};
use futures::StreamExt;
use tarpc::{
    server::{BaseChannel, Channel},
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use tokio::{
    io::AsyncReadExt,
    sync::{RwLock, watch},
};
use tokio_serde::formats::Bincode;
use tracing::{error, info, warn};
use unavi_server_service::{
    ControlService, TrackingIFrame, TrackingPFrame, from_client::StreamHeader,
};
use wtransport::{
    Connection, RecvStream, endpoint::IncomingSession, error::ConnectionError, stream::BiStream,
};
use xdid::{
    core::{did::Did, did_url::DidUrl},
    methods::{key::p256::P256KeyPair, web::reqwest::Url},
};

use crate::{DIRS, session::control::ControlServer};

mod control;
mod init_space_host;
mod pull_transform;
mod transform;
mod voice;

pub const KEY_FRAGMENT: &str = "owner";

const TICKRATE: Duration = Duration::from_millis(50);

type PlayerId = u64;
type SpaceId = String;

#[derive(Clone)]
pub struct SessionSpawner {
    ctx: ServerContext,
    player_id_count: Arc<AtomicU64>,
    pub domain: String,
}

#[derive(Clone)]
struct ServerContext {
    actor: Actor,
    players: Arc<RwLock<HashMap<PlayerId, Player>>>,
    spaces: Arc<RwLock<HashMap<SpaceId, Space>>>,
    player_tickrates: Arc<RwLock<HashMap<PlayerId, HashMap<PlayerId, Duration>>>>,
}

struct Player {
    spaces: HashSet<String>,
    iframe_tx: watch::Sender<(usize, TrackingIFrame)>,
    pframe_tx: watch::Sender<TrackingPFrame>,
}

#[derive(Default)]
struct Space {
    players: HashSet<PlayerId>,
}

#[derive(Clone)]
pub struct SpawnerOptions {
    pub did: Did,
    pub domain: String,
    pub in_memory: bool,
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
                players: Default::default(),
                spaces: Default::default(),
                player_tickrates: Default::default(),
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
            let (iframe_tx, _iframe_rx) = watch::channel((0, Default::default()));
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

        let counts = StreamCounts::default();

        loop {
            match con.accept_uni().await {
                Ok(stream) => {
                    let ctx = self.ctx.clone();
                    let counts = counts.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_stream(ctx, counts, player_id, stream).await {
                            error!("Error handling stream: {e:?}");
                        }
                    });
                }
                Err(e) => {
                    match e {
                        ConnectionError::LocallyClosed | ConnectionError::TimedOut => {
                            info!("Session connection ended: {e}");
                        }
                        e => {
                            warn!("Session connection ended: {e}");
                        }
                    }

                    let mut players = self.ctx.players.write().await;
                    players.remove(&player_id);

                    break Ok(());
                }
            }
        }
    }
}

#[derive(Default, Clone)]
struct StreamCounts {
    transform: Arc<AtomicUsize>,
    voice: Arc<AtomicUsize>,
}

async fn handle_stream(
    ctx: ServerContext,
    counts: StreamCounts,
    player_id: u64,
    mut stream: RecvStream,
) -> anyhow::Result<()> {
    let header_len = stream.read_u16().await? as usize;

    let mut header_buf = vec![0; header_len];
    stream.read_exact(&mut header_buf).await?;

    let (header, _) = bincode::decode_from_slice(&header_buf, bincode::config::standard())?;

    match header {
        StreamHeader::Transform => {
            let count = counts.transform.fetch_add(1, Ordering::SeqCst);

            if count == 0 {
                let res = transform::handle_transform_stream(ctx, player_id, stream).await;
                counts.transform.fetch_sub(1, Ordering::SeqCst);
                res?;
            } else {
                counts.transform.fetch_sub(1, Ordering::SeqCst);
            }
        }
        StreamHeader::Voice => {
            let count = counts.voice.fetch_add(1, Ordering::SeqCst);

            if count == 0 {
                let res = voice::handle_voice_stream(stream).await;
                counts.voice.fetch_sub(1, Ordering::SeqCst);
                res?;
            } else {
                counts.voice.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }

    Ok(())
}
