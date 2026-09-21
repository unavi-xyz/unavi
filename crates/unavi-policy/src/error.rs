use crate::permissions::ApiName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error("permission denied: {0:?}")]
    Permission(ApiName),
    #[error("documents are not in the same space")]
    NotCoPresent,
    #[error("the writer is blocked")]
    Blocked,
    #[error("write to a peer-owned document by a non-owner")]
    NotOwner,
}
