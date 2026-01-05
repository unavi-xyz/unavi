//! ## Sync Protocol
//!
//! Syncs a record and its associated blobs between two nodes.

use std::sync::Arc;

use blake3::Hash;
use iroh::{
    endpoint::{Connection, VarInt},
    protocol::{AcceptError, ProtocolHandler},
};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::warn;

use crate::{SessionToken, StoreContext, sync::combined_stream::CombinedStream};

pub mod client;
mod combined_stream;
mod server;
pub mod shared;

pub const ALPN: &[u8] = b"wds/sync";

// TODO: stream envelopes

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncMsg {
    Begin {
        session: SessionToken,
        record_id: Hash,
        vv: Vec<u8>,
    },
    Envelopes(Vec<Vec<u8>>),
}

#[derive(Debug)]
pub struct SyncProtocol {
    pub(crate) ctx: Arc<StoreContext>,
}

impl SyncProtocol {
    pub const fn new(ctx: Arc<StoreContext>) -> Self {
        Self { ctx }
    }
}

impl ProtocolHandler for SyncProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let stream = connection.accept_bi().await?;
        let combined = CombinedStream(stream.0, stream.1);
        let framed = Framed::new(combined, LengthDelimitedCodec::new());

        let reason = match server::handle_sync(&self.ctx, framed).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Error handling sync: {e:?}");
                "internal error"
            }
        };

        connection.close(VarInt::default(), reason.as_bytes());

        Ok(())
    }
}
