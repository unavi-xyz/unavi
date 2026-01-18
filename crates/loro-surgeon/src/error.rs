use thiserror::Error;

#[derive(Debug, Error)]
pub enum HydrateError {
    #[error("missing field: {0}")]
    MissingField(String),

    #[error("type mismatch at {path}: expected {expected}, got {actual}")]
    TypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("{0}")]
    Custom(String),
}

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("loro error: {0}")]
    Loro(#[from] loro::LoroError),

    #[error("{0}")]
    Custom(String),
}
