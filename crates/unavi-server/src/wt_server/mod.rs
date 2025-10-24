use std::sync::Arc;

use dwn::{Actor, Dwn, document_key::DocumentKey, stores::NativeDbStore};
use futures::StreamExt;
use tarpc::{
    server::{BaseChannel, Channel},
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use tokio::io::AsyncReadExt;
use tokio_serde::formats::Bincode;
use tracing::{error, info};
use unavi_server_service::{
    ControlService,
    from_client::{StreamHeader, TransformMeta},
};
use wtransport::{RecvStream, endpoint::IncomingSession, stream::BiStream};
use xdid::{
    core::{did::Did, did_url::DidUrl},
    methods::{key::p256::P256KeyPair, web::reqwest::Url},
};

use crate::{DIRS, wt_server::control::ControlServer};

mod control;
mod init_world_host;

pub const KEY_FRAGMENT: &str = "owner";

#[derive(Clone)]
pub struct WtServer {
    pub actor: Actor,
    pub domain: String,
}

#[derive(Clone)]
pub struct WtServerOptions {
    pub did: Did,
    pub domain: String,
    pub in_memory: bool,
    pub remote: Url,
    pub vc: P256KeyPair,
}

impl WtServer {
    pub async fn new(opts: WtServerOptions) -> anyhow::Result<Self> {
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
            path_abempty: String::new(),
        };

        let key = Arc::new(key);
        actor.sign_key = Some(key.clone());
        actor.auth_key = Some(key);

        actor.sync().await?;

        Ok(Self {
            actor,
            domain: opts.domain,
        })
    }

    pub async fn handle(self, incoming: IncomingSession) -> anyhow::Result<()> {
        let req = incoming.await?;
        let con = req.accept().await?;

        let stream = con.accept_bi().await?;
        let bi_stream = BiStream::join(stream);
        let framed = Framed::new(bi_stream, LengthDelimitedCodec::default());
        let transport = tarpc::serde_transport::new(framed, Bincode::default());
        let channel = BaseChannel::with_defaults(transport).max_concurrent_requests(2);

        let server = ControlServer::new(self.actor.clone());

        tokio::spawn(channel.execute(server.serve()).for_each(|res| async move {
            tokio::spawn(res);
        }));

        loop {
            let stream = con.accept_uni().await?;

            tokio::spawn(async move {
                if let Err(e) = handle_stream(stream).await {
                    error!("Error handling stream: {e:?}");
                }
            });
        }
    }
}

async fn handle_stream(mut stream: RecvStream) -> anyhow::Result<()> {
    let header_len = stream.read_u16().await? as usize;

    let mut header_buf = vec![0; header_len];
    stream.read_exact(&mut header_buf).await?;

    let (header, _) = bincode::decode_from_slice(&header_buf, bincode::config::standard())?;

    match header {
        StreamHeader::Transform => {
            handle_transform_stream(stream).await?;
        }
        StreamHeader::Voice => {
            handle_voice_stream(stream).await?;
        }
    }

    Ok(())
}

async fn handle_transform_stream(mut stream: RecvStream) -> anyhow::Result<()> {
    let meta_len = stream.read_u16().await? as usize;

    let mut meta_buf = vec![0; meta_len];
    stream.read_exact(&mut meta_buf).await?;

    let (meta, _) = bincode::serde::decode_from_slice::<TransformMeta, _>(
        &meta_buf,
        bincode::config::standard(),
    )?;

    info!("Got transform stream: {meta:?}");

    Ok(())
}

async fn handle_voice_stream(_stream: RecvStream) -> anyhow::Result<()> {
    Ok(())
}
