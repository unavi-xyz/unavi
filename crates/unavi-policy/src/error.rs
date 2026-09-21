use crate::permissions::ApiName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error("permission denied: {0:?}")]
    Permission(ApiName),
    #[error("write to a peer-owned document by a non-owner")]
    NotOwner,
}
