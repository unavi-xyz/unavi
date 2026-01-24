//! ## API Protocol
//!
//! [`irpc`] user API, for use both locally or over the network.

use std::{sync::Arc, time::Duration};

use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointAddr;
use irpc::{
    Client,
    channel::{mpsc, oneshot},
    rpc_requests,
};
use irpc_iroh::IrohProtocol;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{SessionToken, StoreContext, error::ApiError};

mod blob_exists;
mod get_record_pin;
mod pin_blob;
mod pin_record;
mod query_records;
mod read_record;
mod sync_record;
mod upload_blob;
mod upload_envelope;

const MAX_PIN_DURATION: Duration = Duration::from_hours(24 * 90);

pub const ALPN: &[u8] = b"wds/api";

pub(crate) fn protocol(ctx: Arc<StoreContext>) -> (Client<ApiService>, IrohProtocol<ApiService>) {
    let (tx, mut rx) = irpc::channel::mpsc::channel(32);

    n0_future::task::spawn(async move {
        while let Err(e) = handle_requests(&ctx, &mut rx).await {
            error!("Error handling request: {e:?}");
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    (client, IrohProtocol::with_sender(local_sender))
}

#[rpc_requests(message = ApiMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ApiService {
    #[rpc(rx=mpsc::Receiver<Bytes>,tx=oneshot::Sender<Result<Hash, ApiError>>)]
    #[wrap(UploadBlob)]
    UploadBlob { s: SessionToken },
    #[rpc(rx=mpsc::Receiver<Bytes>,tx=oneshot::Sender<Result<(), ApiError>>)]
    #[wrap(UploadEnvelope)]
    UploadEnvelope { s: SessionToken, record_id: Hash },
    #[rpc(tx=oneshot::Sender<Result<(), ApiError>>)]
    #[wrap(PinBlob)]
    PinBlob {
        s: SessionToken,
        hash: Hash,
        expires: i64,
    },
    #[rpc(tx=oneshot::Sender<Result<(), ApiError>>)]
    #[wrap(PinRecord)]
    PinRecord {
        s: SessionToken,
        id: Hash,
        expires: i64,
    },
    #[rpc(tx=oneshot::Sender<Result<Option<i64>, ApiError>>)]
    #[wrap(GetRecordPin)]
    GetRecordPin { s: SessionToken, id: Hash },
    #[rpc(tx=oneshot::Sender<Result<bool, ApiError>>)]
    #[wrap(BlobExists)]
    BlobExists { s: SessionToken, hash: Hash },
    #[rpc(tx=oneshot::Sender<Result<Vec<Hash>, ApiError>>)]
    #[wrap(QueryRecords)]
    QueryRecords {
        s: SessionToken,
        filter: QueryFilter,
    },
    #[rpc(tx=oneshot::Sender<Result<Vec<u8>, ApiError>>)]
    #[wrap(ReadRecord)]
    ReadRecord { s: SessionToken, record_id: Hash },
    #[rpc(tx=oneshot::Sender<Result<(), ApiError>>)]
    #[wrap(SyncRecord)]
    SyncRecord {
        s: SessionToken,
        record_id: Hash,
        remote: EndpointAddr,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryFilter {
    pub creator: Option<String>,
    pub schemas: Vec<Hash>,
}

async fn handle_requests(
    ctx: &Arc<StoreContext>,
    rx: &mut irpc::channel::mpsc::Receiver<ApiMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await? {
        let ctx = Arc::clone(ctx);

        n0_future::task::spawn(async move {
            if let Err(e) = handle_message(ctx, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}

macro_rules! authenticate {
    ($ctx:tt,$inner:tt,$tx:tt) => {
        match $ctx.connections.get_async(&$inner.s).await {
            Some(c) => c.did.clone(),
            None => {
                $tx.send(Err(ApiError::Unauthenticated)).await?;
                return Ok(());
            }
        }
    };
}

pub(crate) use authenticate;

async fn handle_message(ctx: Arc<StoreContext>, msg: ApiMessage) -> anyhow::Result<()> {
    match msg {
        ApiMessage::UploadBlob(channels) => {
            upload_blob::upload_blob(ctx, channels).await?;
        }
        ApiMessage::UploadEnvelope(channels) => {
            upload_envelope::upload_envelope(ctx, channels).await?;
        }
        ApiMessage::PinBlob(channels) => {
            pin_blob::pin_blob(ctx, channels).await?;
        }
        ApiMessage::PinRecord(channels) => {
            pin_record::pin_record(ctx, channels).await?;
        }
        ApiMessage::GetRecordPin(channels) => {
            get_record_pin::get_record_pin(ctx, channels).await?;
        }
        ApiMessage::BlobExists(channels) => {
            blob_exists::blob_exists(ctx, channels).await?;
        }
        ApiMessage::QueryRecords(channels) => {
            query_records::query_records(ctx, channels).await?;
        }
        ApiMessage::ReadRecord(channels) => {
            read_record::read_record(ctx, channels).await?;
        }
        ApiMessage::SyncRecord(channels) => {
            sync_record::sync_record(ctx, channels).await?;
        }
    }

    Ok(())
}
