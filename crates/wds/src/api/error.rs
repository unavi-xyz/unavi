use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors returned by the WDS API.
///
/// These are safe to send across the client/server boundary.
/// Internal details should be logged server-side before converting to these variants.
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum ApiError {
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("record not found")]
    RecordNotFound,
    #[error("access denied")]
    AccessDenied,
    #[error("quota exceeded")]
    QuotaExceeded,
    #[error("blob not found")]
    BlobNotFound,
    #[error("not pinned")]
    NotPinned,
    #[error("sync failed")]
    SyncFailed,
    #[error("internal error")]
    Internal,
}
