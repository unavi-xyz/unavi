//! The one error every host call returns, mirroring `wired:core/error`.

use std::{
    borrow::Cow,
    fmt::Display,
};

use unavi_policy::{
    permissions::{
        HostApi,
        PermissionDenied,
    },
    quota::{
        Flow,
        QuotaError,
        Stock,
    },
};
use unavi_space::replication::cell::SessionError;

/// Why a host call failed.
///
/// Every variant but [`Self::InvalidArgument`] and [`Self::Internal`] carries
/// data rather than a sentence, and those two take a `&'static str` where they
/// can: a hostile script hammers the refusal paths, so a refusal should not
/// allocate.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScriptError {
    #[error("invalid argument: {0}")]
    InvalidArgument(Cow<'static, str>),
    #[error("not found")]
    NotFound,
    #[error("not ready")]
    NotReady,
    #[error("permission denied: {0:?}")]
    Permission(HostApi),
    /// The script may not act on this target, such as a document it does not
    /// own.
    #[error("forbidden")]
    Forbidden,
    #[error("rate limited: {0:?}")]
    RateLimited(Flow),
    #[error("limit reached: {0:?}")]
    LimitReached(Stock),
    #[error("internal: {0}")]
    Internal(Cow<'static, str>),
    /// A handle the guest does not hold. Lowered as a trap, never as a WIT
    /// error.
    #[error("invalid handle")]
    InvalidHandle,
}

impl ScriptError {
    pub fn invalid(detail: impl Into<Cow<'static, str>>) -> Self {
        Self::InvalidArgument(detail.into())
    }

    pub fn internal(err: impl Display) -> Self {
        Self::Internal(err.to_string().into())
    }
}

impl From<QuotaError> for ScriptError {
    fn from(err: QuotaError) -> Self {
        match err {
            QuotaError::Stock(stock) => Self::LimitReached(stock),
            QuotaError::Flow(flow) => Self::RateLimited(flow),
        }
    }
}

impl From<SessionError> for ScriptError {
    fn from(err: SessionError) -> Self {
        match err {
            SessionError::QuotaExceeded => Self::LimitReached(Stock::SessionMemory),
            SessionError::NotOwner => Self::Forbidden,
            SessionError::BadName => Self::invalid("invalid shared property name"),
            SessionError::Unavailable => Self::Internal("the world is gone".into()),
        }
    }
}

impl From<PermissionDenied> for ScriptError {
    fn from(PermissionDenied(api): PermissionDenied) -> Self {
        Self::Permission(api)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quota_error_keeps_the_resource_it_names() {
        assert_eq!(
            ScriptError::from(QuotaError::Stock(Stock::Prims)),
            ScriptError::LimitReached(Stock::Prims)
        );
        assert_eq!(
            ScriptError::from(QuotaError::Flow(Flow::Emit)),
            ScriptError::RateLimited(Flow::Emit)
        );
    }

    #[test]
    fn a_shared_write_by_a_non_author_is_forbidden() {
        assert_eq!(
            ScriptError::from(SessionError::NotOwner),
            ScriptError::Forbidden
        );
    }
}
