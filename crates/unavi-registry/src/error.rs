//! Why a registry refuses a call.

use serde::{
    Deserialize,
    Serialize,
};

/// Why a registry refused a call. Sent over the wire, so variants carry no
/// detail a caller could probe with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum RegistryError {
    #[error("not authenticated")]
    Unauthenticated,
    #[error("not permitted")]
    NotPermitted,
    #[error("malformed payload")]
    Malformed,
    #[error("payload exceeds this registry's limits")]
    TooLarge,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("the caller does not author this namespace")]
    NotAuthor,
    #[error("another identity lists this namespace")]
    NamespaceTaken,
    #[error("already expired")]
    Expired,
    #[error("retention exceeds this registry's maximum")]
    RetentionTooLong,
    #[error("too many submissions held by this identity")]
    TooManySubmissions,
    #[error("present in too many namespaces")]
    TooManyPresences,
    #[error("this namespace is full")]
    NamespaceFull,
    #[error("internal error")]
    Internal,
}
