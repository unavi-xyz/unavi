//! Error types for WDS.
//!
//! Two error types are defined:
//! - [`WdsError`]: Internal error type with full context.
//! - [`ApiError`]: External error type safe for client/server boundary.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Internal error type for WDS operations.
///
/// Contains full error context for debugging and logging.
/// Convert to [`ApiError`] before sending to clients.
#[derive(Debug, Error)]
pub enum WdsError {
    // Auth/identity errors.
    #[error("invalid signature")]
    InvalidSignature,
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("DID resolution failed: {0}")]
    DidResolution(String),

    // Access control.
    #[error("access denied")]
    AccessDenied,
    #[error("record not pinned")]
    NotPinned,
    #[error("quota exceeded")]
    QuotaExceeded,

    // Resource errors.
    #[error("record not found")]
    RecordNotFound,
    #[error("blob not found")]
    BlobNotFound,

    // Validation.
    #[error("schema validation failed: {0}")]
    SchemaValidation(String),

    // Infrastructure.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] postcard::Error),
    #[error("sync protocol error")]
    SyncFailed,

    // Fallback.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// External error type for the WDS API.
///
/// Safe to send across the client/server boundary.
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
    #[error("invalid signature")]
    InvalidSignature,
    #[error("internal error")]
    Internal,
}

impl WdsError {
    /// Convert to [`ApiError`], recovering wrapped errors from [`WdsError::Other`].
    fn to_api_error(&self) -> ApiError {
        match self {
            Self::InvalidSignature => ApiError::InvalidSignature,
            Self::Unauthenticated => ApiError::Unauthenticated,
            Self::AccessDenied => ApiError::AccessDenied,
            Self::NotPinned => ApiError::NotPinned,
            Self::QuotaExceeded => ApiError::QuotaExceeded,
            Self::RecordNotFound => ApiError::RecordNotFound,
            Self::BlobNotFound => ApiError::BlobNotFound,
            Self::SyncFailed => ApiError::SyncFailed,
            Self::SchemaValidation(msg) if msg.contains("access denied") => ApiError::AccessDenied,
            Self::Other(e) => {
                // Try to recover WdsError that was wrapped via .into().
                if let Some(inner) = e.downcast_ref::<Self>() {
                    return inner.to_api_error();
                }
                ApiError::Internal
            }
            Self::DidResolution(_)
            | Self::Database(_)
            | Self::Serialization(_)
            | Self::SchemaValidation(_) => ApiError::Internal,
        }
    }
}

impl From<WdsError> for ApiError {
    fn from(err: WdsError) -> Self {
        err.to_api_error()
    }
}
