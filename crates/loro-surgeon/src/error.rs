#[derive(Debug, thiserror::Error)]
pub enum HydrateError {
    #[error(transparent)]
    Loro(#[from] loro::LoroError),

    #[error("expected {expected}, found {found}")]
    Unexpected {
        expected: &'static str,
        found: &'static str,
    },

    #[error("missing required property: {key}")]
    Missing { key: String },

    #[error("json deserialization failed for {key}: {source}")]
    Json {
        key: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("integer overflow: {value} doesn't fit in {target_type}")]
    Overflow {
        value: i64,
        target_type: &'static str,
    },
}

impl HydrateError {
    #[must_use]
    pub const fn unexpected(expected: &'static str, found: &'static str) -> Self {
        Self::Unexpected { expected, found }
    }

    pub fn missing(key: impl Into<String>) -> Self {
        Self::Missing { key: key.into() }
    }
}

pub trait HydrateResultExt<T> {
    fn strip_unexpected(self) -> Result<Option<T>, HydrateError>;
}

impl<T> HydrateResultExt<T> for Result<T, HydrateError> {
    fn strip_unexpected(self) -> Result<Option<T>, HydrateError> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(HydrateError::Unexpected { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReconcileError {
    #[error(transparent)]
    Loro(#[from] loro::LoroError),

    #[error("json serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("type mismatch: expected {expected} container, found {found}")]
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
    },
}
