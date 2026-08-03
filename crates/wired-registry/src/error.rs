use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Copy, thiserror::Error, Serialize, Deserialize)]
pub enum RegistryError {
    #[error("not authenticated")]
    Unauthenticated,
    #[error("not permitted")]
    NotPermitted,
    #[error("malformed payload")]
    Malformed,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("already expired")]
    Expired,
    #[error("retention exceeds this registry's maximum")]
    RetentionTooLong,
    #[error("too many submissions held by this identity")]
    TooManySubmissions,
    #[error("internal error")]
    Internal,
}
