use crate::{
    document::ApiName,
    trust::Trust,
};

/// A denial by one of the policy layers.
///
/// Carries what was denied rather than a rendered sentence: the guest-facing
/// variant has no payload, so a formatted message would be allocated on every
/// refused call and then dropped at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error("permission denied: {0:?}")]
    Permission(ApiName),
    #[error("documents are not in the same space")]
    NotCoPresent,
    #[error("writes need {required:?}, caller is {actual:?}")]
    Rung { required: Trust, actual: Trust },
    #[error("write to a peer-owned document by a non-owner")]
    NotOwner,
}

impl PolicyError {
    /// Whether this is a missing permission rather than a write out of reach.
    /// The two are separate variants in `wired:error`.
    #[must_use]
    pub const fn is_permission(self) -> bool {
        matches!(self, Self::Permission(_))
    }
}
