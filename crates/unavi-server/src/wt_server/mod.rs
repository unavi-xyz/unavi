use std::sync::Arc;

use dwn::{Actor, Dwn, document_key::DocumentKey, stores::NativeDbStore};
use futures::StreamExt;
use tarpc::{
    server::{BaseChannel, Channel},
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use tokio_serde::formats::Bincode;
use unavi_constants::REMOTE_DWN_URL;
use unavi_server_service::ControlService;
use xdid::{
    core::{did::Did, did_url::DidUrl},
    methods::{key::p256::P256KeyPair, web::reqwest::Url},
};
use xwt_wtransport::wtransport::{endpoint::IncomingSession, stream::BiStream};

use crate::{DIRS, wt_server::control::ControlServer};

mod control;
mod init_world_host;

pub const KEY_FRAGMENT: &str = "owner";

#[derive(Clone)]
pub struct WtServer {
    pub actor: Actor,
    pub domain: String,
}

impl WtServer {
    pub async fn new(did: Did, vc: P256KeyPair, domain: String) -> anyhow::Result<Self> {
        let store = {
            let mut path = DIRS.data_local_dir().to_path_buf();
            path.push("data.db");
            NativeDbStore::new(path)?
        };
        let dwn = Dwn::from(store);

        let mut actor = Actor::new(did.clone(), dwn);

        let remote_url = Url::parse(REMOTE_DWN_URL)?;
        actor.remote = Some(remote_url);

        let mut key: DocumentKey = vc.into();
        key.url = DidUrl {
            did,
            query: None,
            fragment: Some(KEY_FRAGMENT.to_string()),
            path_abempty: String::new(),
        };

        let key = Arc::new(key);
        actor.sign_key = Some(key.clone());
        actor.auth_key = Some(key);

        actor.sync().await?;

        Ok(Self { actor, domain })
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
