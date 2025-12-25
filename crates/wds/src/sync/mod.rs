//! ## Sync Protocol
//!
//! Syncs a record and its associated blobs between two nodes.

use std::sync::Arc;

use blake3::Hash;
use iroh::{
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::warn;

use crate::{SessionToken, StoreContext, sync::combined_stream::CombinedStream};

pub mod client;
mod combined_stream;
mod server;
mod shared;

pub const ALPN: &[u8] = b"wds/sync";

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncMsg {
    Begin {
        session: SessionToken,
        record_id: Hash,
        vv: Vec<u8>,
    },
    Envelopes(Vec<Vec<u8>>),
    Done,
}

#[derive(Debug)]
pub struct SyncProtocol {
    pub ctx: Arc<StoreContext>,
}

impl ProtocolHandler for SyncProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let stream = connection.accept_bi().await?;
        let combined = CombinedStream(stream.0, stream.1);
        let framed = Framed::new(combined, LengthDelimitedCodec::new());

        if let Err(e) = server::handle_sync(&self.ctx, framed).await {
            warn!("Error handling sync: {e:?}");
        }

        Ok(())
    }
}
