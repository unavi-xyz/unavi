use std::sync::Arc;

use dwn::{Actor, Dwn, document_key::DocumentKey, stores::NativeDbStore};
use futures::StreamExt;
use tarpc::{
    server::{BaseChannel, Channel},
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use tokio_serde::formats::Bincode;
use tracing::info;
use unavi_server_service::ControlService;
use xdid::methods::{
    key::{DidKeyPair, PublicKey},
    web::reqwest::Url,
};
use xwt_wtransport::wtransport::{endpoint::IncomingSession, stream::BiStream};

use crate::{DIRS, server::control::ControlServer};

mod control;
mod init_world_host;
mod key_pair;

const REMOTE_DWN_URL: &str = "http://localhost:8080";

#[derive(Clone)]
pub struct Server {
    pub actor: Actor,
}

impl Server {
    pub async fn new() -> anyhow::Result<Self> {
        let store = {
            let mut path = DIRS.data_local_dir().to_path_buf();
            path.push("data.db");
            NativeDbStore::new(path)?
        };
        let dwn = Dwn::from(store);

        let pair = key_pair::get_or_create_key()?;

        let did = pair.public().to_did();
        info!("Loaded identity: {did}");

        let mut actor = Actor::new(did, dwn);

        let remote_url = Url::parse(REMOTE_DWN_URL)?;
        actor.remote = Some(remote_url);

        let key = Arc::<DocumentKey>::new(pair.into());
        actor.sign_key = Some(key.clone());
        actor.auth_key = Some(key);

        actor.sync().await?;

        init_world_host::init_world_host(&actor).await?;

        Ok(Self { actor })
    }

    pub async fn handle(self, incoming: IncomingSession) -> anyhow::Result<()> {
        let req = incoming.await?;
        let con = req.accept().await?;

        let ctrl_task = {
            let stream = con.accept_bi().await?;
            let bi_stream = BiStream::join(stream);
            let framed = Framed::new(bi_stream, LengthDelimitedCodec::default());
            let transport = tarpc::serde_transport::new(framed, Bincode::default());
            let channel = BaseChannel::with_defaults(transport).max_concurrent_requests(2);

            let server = ControlServer { actor: self.actor };

            tokio::spawn(channel.execute(server.serve()).for_each(|res| async move {
                tokio::spawn(res);
            }))
        };

        ctrl_task.await?;

        Ok(())
    }
}
