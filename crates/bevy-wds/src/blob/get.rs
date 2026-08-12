use std::time::Duration;

use async_channel::Sender;
use bevy::{
    prelude::*,
    tasks::futures_lite::StreamExt,
};
use blake3::Hash;
use bytes::Bytes;
use iroh_blobs::api::blobs::Blobs;
use thiserror::Error;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;

use crate::LocalBlobs;

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
    tx: Sender<Result<Bytes, BlobError>>,
    blobs: Blobs,
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
            res = tokio::time::timeout(ATTEMPT_TIMEOUT, get_blob(hash, &blobs)) => res,
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

/// Bounds reads of both a fetch and a cached blob: the cached copy may have
/// been written by a path that did not bound it.
async fn get_blob(hash: Hash, blobs: &Blobs) -> Result<Bytes, BlobError> {
    if blobs.has(hash).await.map_err(BlobError::from_std)? {
        return read_bounded(hash, blobs).await;
    }

    let mut stream = blobs
        .observe(hash)
        .stream()
        .await
        .map_err(BlobError::from_std)?;

    while let Some(field) = stream.next().await {
        let size = field.size();

        if size >= MAX_SIZE {
            return Err(BlobError::TooLarge { size });
        }

        if field.state().complete {
            break;
        }

        if size > 0 {
            let val = field.state().validated_size.unwrap_or_default();
            let progress = val as f64 / size as f64;
            info!(hash = %hash, "Downloading: {:.2}%", progress * 100.0);
        }
    }

    read_bounded(hash, blobs).await
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
    fn only_io_errors_are_retryable() {
        assert!(BlobError::Io(anyhow::anyhow!("network blip")).retryable());
        assert!(!BlobError::TooLarge { size: 1 }.retryable());
        assert!(!BlobError::Exhausted { attempts: 3 }.retryable());
    }
}
