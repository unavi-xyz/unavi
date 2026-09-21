use crate::{
    permissions::ApiName,
    trust::Trust,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error("permission denied: {0:?}")]
    Permission(ApiName),
    #[error("documents are not in the same space")]
    NotCoPresent,
    #[error("writes need trust {required:?}, caller is {actual:?}")]
    Threshold { required: Trust, actual: Trust },
    #[error("write to a peer-owned document by a non-owner")]
    NotOwner,
}
