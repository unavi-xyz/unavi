use anyhow::bail;
use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use blake3::Hash;
use bytes::Bytes;
use tokio::sync::{mpsc::Sender, oneshot};
use unavi_util::async_task::spawn_async_task;
use wds::Blobs;

use crate::LocalBlobs;

#[derive(Event)]
pub struct GetBlob {
    pub hash: Hash,
    pub cancel: Option<oneshot::Receiver<()>>,
    pub tx: Sender<Bytes>,
}

pub(crate) fn on_get_blob(mut req: On<GetBlob>, blobs: Query<&LocalBlobs>) {
    let Ok(blobs) = blobs.single().map(|x| x.0.clone()) else {
        warn!("Unable to get blob: no LocalBlobs");
        return;
    };

    let event = req.event_mut();
    let hash = event.hash;
    let cancel = event.cancel.take();
    let tx = event.tx.clone();

    spawn_async_task(async move {
        if let Err(err) = inner(hash, cancel, tx, blobs).await {
            error!(?err, "Failed to get blob");
        }
    });
}

async fn inner(
    hash: Hash,
    cancel: Option<oneshot::Receiver<()>>,
    tx: Sender<Bytes>,
    blobs: Blobs,
) -> anyhow::Result<()> {
    let cancel_fut = async move {
        match cancel {
            Some(rx) => {
                rx.await.ok();
            }
            None => std::future::pending::<()>().await,
        }
    };
    tokio::select! {
        () = cancel_fut => {},
        res = get_blob(hash, blobs) => {
            tx.send(res?).await?;
        }
    }
    Ok(())
}

const MB: u64 = 1024 * 1024;
const MAX_SIZE: u64 = 512 * MB;

async fn get_blob(hash: Hash, blobs: Blobs) -> anyhow::Result<Bytes> {
    if blobs.has(hash).await? {
        let res = blobs.get_bytes(hash).await?;
        return Ok(res);
    }

    let mut stream = blobs.observe(hash).stream().await?;

    while let Some(field) = stream.next().await {
        let size = field.size();

        if size >= MAX_SIZE {
            bail!("blob too large: {size}");
        }

        if field.state().complete {
            break;
        }

        let val = field.state().validated_size.unwrap_or_default();
        let progress = val as f64 / size as f64;
        info!(hash = %hash, "Downloading: {:.2}%", progress * 100.0);
    }

    let res = blobs.get_bytes(hash).await?;
    Ok(res)
}
