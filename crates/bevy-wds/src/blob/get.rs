use std::sync::Arc;

use anyhow::bail;
use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use blake3::Hash;
use bytes::Bytes;
use tokio::sync::{Notify, mpsc::Sender};
use wds::Blobs;

use crate::LocalBlobs;

#[derive(Event, Clone)]
pub struct GetBlob {
    pub hash: Hash,
    pub cancel: Arc<Notify>,
    pub tx: Sender<Bytes>,
}

pub(crate) fn on_get_blob(req: On<GetBlob>, blobs: Query<&LocalBlobs>) {
    let Ok(blobs) = blobs.single().map(|x| x.0.clone()) else {
        warn!("Unable to get blob: no LocalBlobs");
        return;
    };

    let event = req.event().clone();

    unavi_wasm_compat::spawn_thread(async move {
        if let Err(err) = inner(event, blobs).await {
            error!(?err, "Failed to get blob");
        }
    });
}

async fn inner(event: GetBlob, blobs: Blobs) -> anyhow::Result<()> {
    tokio::select! {
        () = event.cancel.notified() => {},
        res = get_blob(&event, blobs) => {
            event.tx.send(res?).await?;
        }
    }
    Ok(())
}

const MB: u64 = 1024 * 1024;
const MAX_SIZE: u64 = 512 * MB;

async fn get_blob(event: &GetBlob, blobs: Blobs) -> anyhow::Result<Bytes> {
    if blobs.has(event.hash).await? {
        let res = blobs.get_bytes(event.hash).await?;
        return Ok(res);
    }

    let mut stream = blobs.observe(event.hash).stream().await?;

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
        info!(hash = %event.hash, "Downloading: {:.2}%", progress * 100.0);
    }

    let res = blobs.get_bytes(event.hash).await?;
    Ok(res)
}
