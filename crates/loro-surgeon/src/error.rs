//! Error types for hydration and reconciliation.

use thiserror::Error;

/// Error returned when hydrating a value from a Loro document.
#[derive(Debug, Error)]
pub enum HydrateError {
    /// A required field was missing from the Loro map.
    #[error("missing field: {0}")]
    MissingField(String),

    /// The Loro value type did not match the expected Rust type.
    #[error("type mismatch at {path}: expected {expected}, got {actual}")]
    TypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    /// A custom error from a user-defined hydration function.
    #[error("{0}")]
    Custom(String),
}

/// Error returned when reconciling a value to a Loro document.
#[derive(Debug, Error)]
pub enum ReconcileError {
    /// An error from the Loro library.
    #[error("loro error: {0}")]
    Loro(#[from] loro::LoroError),

    /// A custom error from a user-defined reconciliation function.
    #[error("{0}")]
    Custom(String),
}
