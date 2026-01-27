use anyhow::bail;
use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use bytes::Bytes;
use wds::Blobs;

use crate::{AwaitBlob, LocalBlobs};

pub fn handle_await_blob(req: On<AwaitBlob>, blobs: Query<&LocalBlobs>) {
    let Ok(blobs) = blobs.single().map(|x| x.0.clone()) else {
        warn!("Unable to handle blob request: no LocalBlobs");
        return;
    };

    let event = req.event().clone();

    unavi_wasm_compat::spawn_thread(async move {
        if let Err(err) = inner(event, blobs).await {
            error!(?err, "failed to handle blob request");
        }
    });
}

async fn inner(event: AwaitBlob, blobs: Blobs) -> anyhow::Result<()> {
    tokio::select! {
        () = event.cancel.notified() => {},
        res = get_or_await_bytes(&event, blobs) => {
            event.tx.send(res?).await?;
        }
    }

    Ok(())
}

const MB: u64 = 1024 * 1024;
const MAX_SIZE: u64 = 512 * MB;

async fn get_or_await_bytes(event: &AwaitBlob, blobs: Blobs) -> anyhow::Result<Bytes> {
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
        info!(hash = %event.hash, "downloading: {:.2}%", progress * 100.0);
    }

    let res = blobs.get_bytes(event.hash).await?;
    Ok(res)
}
