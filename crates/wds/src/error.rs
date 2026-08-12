//! [`ApiError`]: WDS's error type safe for the client/server boundary.

use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

/// External error type for the WDS API, safe to send across the client/server
/// boundary.
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum ApiError {
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("access denied")]
    AccessDenied,
    #[error("quota exceeded")]
    QuotaExceeded,
    #[error("blob not found")]
    BlobNotFound,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("internal error")]
    Internal,
}
