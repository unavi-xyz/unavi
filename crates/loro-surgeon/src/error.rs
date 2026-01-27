use smol_str::SmolStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HydrateError {
    #[error("missing field: {0}")]
    MissingField(SmolStr),

    #[error("type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: SmolStr, actual: SmolStr },

    #[error("{0}")]
    Custom(SmolStr),
}

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("loro error: {0}")]
    Loro(#[from] loro::LoroError),

    #[error("{0}")]
    Custom(SmolStr),
}
