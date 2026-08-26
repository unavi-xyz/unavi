use std::time::Duration;

use async_channel::Sender;
use bevy::{
    prelude::*,
    tasks::futures_lite::StreamExt,
};
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use iroh_blobs::api::{
    Store as BlobStore,
    blobs::Blobs,
    downloader::Downloader,
};
use thiserror::Error;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;

use crate::{
    BlobProviders,
    LocalBlobStore,
    LocalBlobs,
    LocalDownloader,
    SyncTargets,
};

const MB: u64 = 1024 * 1024;
/// Every read here lands the whole blob in memory at once, so this bounds a
/// single allocation rather than a transfer.
const MAX_SIZE: u64 = 64 * MB;
/// Cap on one download attempt, so a wedged sync cannot stall a fetch forever.
const ATTEMPT_TIMEOUT: Duration = Duration::from_mins(5);
/// First retry waits this long; each attempt doubles until `MAX_RETRY_DELAY`.
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRY_DELAY: Duration = Duration::from_mins(1);
const MAX_ATTEMPTS: u32 = 10;
/// Progress is logged once per step, so a large transfer emits a handful of
/// lines rather than one per chunk.
const PROGRESS_STEP: f64 = 0.1;

fn progress_step(progress: f64) -> u32 {
    (progress / PROGRESS_STEP) as u32
}

/// The backoff before attempt `attempt` (0-indexed), doubling to a cap.
fn retry_delay(attempt: u32) -> Duration {
    Duration::from_secs(
        INITIAL_RETRY_DELAY
            .as_secs()
            .checked_shl(attempt)
            .unwrap_or(u64::MAX)
            .min(MAX_RETRY_DELAY.as_secs()),
    )
}

#[derive(Debug, Error)]
pub enum BlobError {
    #[error("blob too large: {size} bytes")]
    TooLarge { size: u64 },
    #[error("fetch failed after {attempts} attempts")]
    Exhausted { attempts: u32 },
    #[error("fetch failed: {0}")]
    Io(#[from] anyhow::Error),
}

impl BlobError {
    /// Permanent failures abort immediately; only network-style failures retry.
    pub(crate) const fn retryable(&self) -> bool {
        matches!(self, Self::Io(_))
    }

    /// Wraps a library error that `#[from] anyhow::Error` does not cover.
    fn from_std(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Io(anyhow::Error::new(err))
    }
}

#[derive(Event)]
pub struct GetBlob {
    pub hash:   Hash,
    pub cancel: Option<oneshot::Receiver<()>>,
    pub tx:     Sender<Result<Bytes, BlobError>>,
}

pub(crate) fn on_get_blob(
    mut req: On<GetBlob>,
    blobs: Query<&LocalBlobs>,
    stores: Query<&LocalBlobStore>,
    downloaders: Query<&LocalDownloader>,
    targets: Query<&SyncTargets>,
    peers: Query<&BlobProviders>,
) {
    let Ok(blobs) = blobs.single().map(|x| x.0.clone()) else {
        warn!("Unable to get blob: no LocalBlobs");
        return;
    };

    // The tags a fetch roots itself with live on the store, not on the blobs
    // client, so a fetch cannot run without it.
    let Ok(store) = stores.single().map(|x| x.0.clone()) else {
        warn!("Unable to get blob: no LocalBlobStore");
        return;
    };

    let downloader = downloaders.single().ok().map(|x| x.0.clone());
    let mut providers = targets
        .iter()
        .flat_map(|x| x.0.iter().map(|actor| actor.host().id))
        .collect::<Vec<_>>();
    // Appended rather than merged, so a configured server is still tried first.
    for id in peers.iter().flat_map(|x| x.0.iter().copied()) {
        if !providers.contains(&id) {
            providers.push(id);
        }
    }

    let event = req.event_mut();
    let hash = event.hash;
    let cancel = event.cancel.take();
    let tx = event.tx.clone();

    spawn_async_task(async move {
        if let Err(err) = inner(hash, cancel, tx, blobs, store, downloader, providers).await {
            error!(?err, "Failed to get blob");
        }
    });
}

async fn inner(
    hash: Hash,
    cancel: Option<oneshot::Receiver<()>>,
    tx: Sender<Result<Bytes, BlobError>>,
    blobs: Blobs,
    store: BlobStore,
    downloader: Option<Downloader>,
    providers: Vec<EndpointId>,
) -> anyhow::Result<()> {
    let mut cancel = Box::pin(async move {
        match cancel {
            Some(rx) => {
                rx.await.ok();
            }
            None => std::future::pending::<()>().await,
        }
    });

    for attempt in 0..MAX_ATTEMPTS {
        let res = tokio::select! {
            () = &mut cancel => return Ok(()),
            res = n0_future::time::timeout(
                ATTEMPT_TIMEOUT,
                get_blob(hash, &blobs, &store, downloader.as_ref(), &providers),
            ) => res,
        };
        match res {
            Ok(Ok(bytes)) => {
                tx.send(Ok(bytes)).await?;
                return Ok(());
            }
            Ok(Err(err)) => {
                if !err.retryable() {
                    tx.send(Err(err)).await?;
                    return Ok(());
                }
                warn!(%hash, attempt, ?err, "blob fetch failed, retrying");
            }
            Err(_) => {
                warn!(%hash, attempt, "blob fetch attempt timed out");
            }
        }
        n0_future::time::sleep(retry_delay(attempt)).await;
    }

    tx.send(Err(BlobError::Exhausted {
        attempts: MAX_ATTEMPTS,
    }))
    .await?;
    Ok(())
}

/// Bounds the read of a fetched blob and of an already cached copy, which may
/// have been written by a path that did not bound it.
async fn get_blob(
    hash: Hash,
    blobs: &Blobs,
    store: &BlobStore,
    downloader: Option<&Downloader>,
    providers: &[EndpointId],
) -> Result<Bytes, BlobError> {
    wds::cache::touch(store, hash, wds::cache::DEFAULT_TTL).await?;

    if blobs.has(hash).await.map_err(BlobError::from_std)? {
        return read_bounded(hash, blobs).await;
    }

    match downloader {
        Some(downloader) if !providers.is_empty() => {
            // Watching alongside the download keeps the size bound enforced
            // mid-transfer, and cancels a download that outgrows it.
            tokio::select! {
                res = pull(downloader, hash, providers) => res?,
                res = watch_until_complete(hash, blobs) => res?,
            }
        }
        // Content the doc engine pulls arrives without a provider list.
        _ => watch_until_complete(hash, blobs).await?,
    }

    read_bounded(hash, blobs).await
}

async fn pull(
    downloader: &Downloader,
    hash: Hash,
    providers: &[EndpointId],
) -> Result<(), BlobError> {
    downloader
        .download(iroh_blobs::Hash::from(hash), providers.to_vec())
        .await
        .map_err(|err| BlobError::Io(anyhow::Error::from(err)))
}

async fn watch_until_complete(hash: Hash, blobs: &Blobs) -> Result<(), BlobError> {
    let mut stream = blobs
        .observe(hash)
        .stream()
        .await
        .map_err(BlobError::from_std)?;

    let mut logged = 0;

    while let Some(field) = stream.next().await {
        let size = field.size();

        if size >= MAX_SIZE {
            return Err(BlobError::TooLarge { size });
        }

        if field.is_complete() {
            return Ok(());
        }

        if size > 0 {
            // `validated_size` only resolves once the final chunk lands, so
            // received bytes are what a transfer's progress reads from.
            let progress = field.total_bytes() as f64 / size as f64;
            let step = progress_step(progress);
            if step > logged {
                logged = step;
                info!(hash = %hash, "Downloading: {:.0}%", progress * 100.0);
            }
        }
    }

    Ok(())
}

async fn read_bounded(hash: Hash, blobs: &Blobs) -> Result<Bytes, BlobError> {
    let size = blobs
        .observe(hash)
        .await
        .map_err(BlobError::from_std)?
        .size();
    if size >= MAX_SIZE {
        return Err(BlobError::TooLarge { size });
    }
    blobs.get_bytes(hash).await.map_err(BlobError::from_std)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_delay_doubles_then_caps() {
        assert_eq!(retry_delay(0), Duration::from_secs(1));
        assert_eq!(retry_delay(1), Duration::from_secs(2));
        assert_eq!(retry_delay(2), Duration::from_secs(4));
        assert_eq!(retry_delay(30), MAX_RETRY_DELAY);
    }

    #[test]
    fn progress_only_steps_forward_in_tenths() {
        assert_eq!(progress_step(0.0), 0);
        assert_eq!(progress_step(0.0999), 0, "a chunk-sized delta logs nothing");
        assert_eq!(progress_step(0.1), 1);
        assert_eq!(progress_step(0.55), 5);
        assert_eq!(progress_step(1.0), 10);
    }

    #[test]
    fn only_io_errors_are_retryable() {
        assert!(BlobError::Io(anyhow::anyhow!("network blip")).retryable());
        assert!(!BlobError::TooLarge { size: 1 }.retryable());
        assert!(!BlobError::Exhausted { attempts: 3 }.retryable());
    }
}
