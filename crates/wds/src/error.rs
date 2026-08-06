//! [`ApiError`]: WDS's error type safe for the client/server boundary.

use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

/// External error type for the WDS API.
///
/// Safe to send across the client/server boundary.
/// Internal details should be logged server-side before converting to these
/// variants.
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
