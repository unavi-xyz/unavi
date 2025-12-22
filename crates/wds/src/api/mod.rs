//! [`irpc`] WDS API, for use both locally or as an `iroh` protocol.

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use futures::{StreamExt, TryStreamExt};
use irpc::{
    Client, WithChannels,
    channel::{mpsc, oneshot},
    rpc_requests,
};
use irpc_iroh::IrohProtocol;
use loro::LoroDoc;
use n0_error::Meta;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use time::OffsetDateTime;
use tracing::error;

use crate::ConnectionState;

pub const ALPN: &[u8] = b"wds/api";

pub fn protocol(conn: Arc<ConnectionState>) -> IrohProtocol<ApiService> {
    let (tx, mut rx) = irpc::channel::mpsc::channel(8);

    tokio::task::spawn(async move {
        while let Err(e) = handle_requests(&conn, &mut rx).await {
            error!("Error handling request: {e:?}");
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    IrohProtocol::with_sender(local_sender)
}

#[rpc_requests(message = ApiMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ApiService {
    #[rpc(tx=oneshot::Sender<Result<blake3::Hash, SmolStr>>)]
    #[wrap(CreateRecord)]
    CreateRecord,
    #[rpc(rx=mpsc::Receiver<Bytes>,tx=oneshot::Sender<Result<(), SmolStr>>)]
    #[wrap(UploadBlob)]
    UploadBlob {
        hash: blake3::Hash,
        byte_length: usize,
    },
    #[rpc(tx=oneshot::Sender<Result<(), SmolStr>>)]
    #[wrap(PinBlob)]
    PinBlob { id: blake3::Hash, expires: u64 },
    #[rpc(tx=oneshot::Sender<Result<(), SmolStr>>)]
    #[wrap(PinRecord)]
    PinRecord { id: blake3::Hash, expires: u64 },
}

async fn handle_requests(
    conn: &Arc<ConnectionState>,
    rx: &mut irpc::channel::mpsc::Receiver<ApiMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await? {
        let conn = Arc::clone(conn);

        tokio::spawn(async move {
            if let Err(e) = handle_message(conn, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}

macro_rules! authenticate {
    ($conn:tt,$tx:tt) => {
        match $conn.authentication.get() {
            Some(did) => did,
            None => {
                $tx.send(Err("unauthenticated".into())).await?;
                return Ok(());
            }
        }
    };
}

const DEFAULT_BLOB_TTL: Duration = Duration::from_hours(1);

async fn handle_message(conn: Arc<ConnectionState>, msg: ApiMessage) -> anyhow::Result<()> {
    match msg {
        ApiMessage::CreateRecord(WithChannels { tx, .. }) => {
            let _did = authenticate!(conn, tx);

            let _doc = LoroDoc::new();

            todo!()
        }
        ApiMessage::UploadBlob(WithChannels { inner, tx, rx, .. }) => {
            let did = authenticate!(conn, tx);

            let total_bytes = Arc::new(AtomicUsize::default());
            let target_len = inner.byte_length;

            let stream = {
                let total_bytes = Arc::clone(&total_bytes);
                rx.into_stream()
                    .map(move |res| {
                        if let Ok(b) = &res {
                            let len = total_bytes.fetch_add(b.len(), Ordering::Release);
                            if len > target_len {
                                return Err(
                                    irpc::channel::mpsc::RecvError::MaxMessageSizeExceeded {
                                        meta: Meta::default(),
                                    },
                                );
                            }
                        }
                        res
                    })
                    .map_err(|_| std::io::ErrorKind::Other.into())
            };

            let temp_tag = conn.blob_store.add_stream(stream).await.temp_tag().await?;

            if temp_tag.hash().as_bytes() != inner.hash.as_bytes() {
                tx.send(Err("stored hash does not equal specified hash".into()))
                    .await?;
                return Ok(());
            }

            let blob_len = total_bytes.load(Ordering::Acquire);
            if blob_len != target_len {
                tx.send(Err("invalid byte length".into())).await?;
                return Ok(());
            }

            let blob_len = i64::try_from(blob_len)?;

            let db = conn.db.pool();
            let mut db_tx = db.begin().await?;

            let did_str = did.to_string();

            let update_res = sqlx::query!(
                "UPDATE user_quotas
                 SET bytes_used = bytes_used + ?
                 WHERE owner = ? AND bytes_used + ? <= quota_bytes",
                blob_len,
                did_str,
                blob_len
            )
            .execute(&mut *db_tx)
            .await?;

            if update_res.rows_affected() == 0 {
                // Quota exceeded (or user doesn't exist).
                tx.send(Err("quota exceeded".into())).await?;
                return Ok(());
            }

            let expires = (OffsetDateTime::now_utc() + DEFAULT_BLOB_TTL).unix_timestamp();
            let tag_name = format!("{did}_{expires}");

            sqlx::query!(
                "INSERT INTO blobs (tag, creator, size) VALUES (?, ?, ?)",
                tag_name,
                did_str,
                blob_len
            )
            .execute(&mut *db_tx)
            .await?;

            db_tx.commit().await?;

            // Only persist blob tag after tracking blob in DB.
            // TODO We could end up with DB tracking but no blob tag if we crash or error.
            conn.blob_store.tags().set(tag_name, temp_tag).await?;
        }
        ApiMessage::PinBlob(WithChannels { inner: _, tx, .. }) => {
            let _did = authenticate!(conn, tx);
            todo!()
        }
        ApiMessage::PinRecord(WithChannels { inner: _, tx, .. }) => {
            let _did = authenticate!(conn, tx);
            todo!()
        }
    }

    Ok(())
}
