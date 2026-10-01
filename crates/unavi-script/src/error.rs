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
use unavi_space::state::cell::SessionError;

/// Host-side canonical error, mirroring `wired:error/types.error`.
///
/// Every variant but `Other` carries structured data rather than a rendered
/// sentence: the matching WIT variants have no payload, so a formatted message
/// would be allocated on every refused call and dropped at the boundary — on
/// the one path a hostile script is expected to hammer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScriptError {
    #[error("{0}")]
    Other(String),
    /// A rate limit, which refills on its own: a retry may succeed.
    #[error("rate limit exceeded: {0:?}")]
    QuotaFlow(Flow),
    /// A ceiling on a held resource, which frees only when something else
    /// releases: a retry without freeing will not.
    #[error("resource ceiling reached: {0:?}")]
    QuotaStock(Stock),
    /// The document's author is not trusted for this API.
    #[error("permission denied: {0:?}")]
    Permission(HostApi),
    /// A write to a peer-owned document by a peer that does not own it.
    #[error("write to a peer-owned document by a non-owner")]
    NotOwner,
}

impl ScriptError {
    pub fn other(detail: impl Into<String>) -> Self {
        Self::Other(detail.into())
    }
}

impl From<QuotaError> for ScriptError {
    fn from(err: QuotaError) -> Self {
        match err {
            QuotaError::Stock(stock) => Self::QuotaStock(stock),
            QuotaError::Flow(flow) => Self::QuotaFlow(flow),
        }
    }
}

impl From<SessionError> for ScriptError {
    fn from(err: SessionError) -> Self {
        match err {
            SessionError::QuotaExceeded => Self::QuotaStock(Stock::SessionMemory),
            SessionError::NotOwner => Self::NotOwner,
            SessionError::BadName | SessionError::Other => Self::Other(err.to_string()),
        }
    }
}

impl From<PermissionDenied> for ScriptError {
    fn from(PermissionDenied(api): PermissionDenied) -> Self {
        Self::Permission(api)
    }
}

impl From<anyhow::Error> for ScriptError {
    fn from(err: anyhow::Error) -> Self {
        let err = match err.downcast::<QuotaError>() {
            Ok(quota) => return quota.into(),
            Err(err) => err,
        };
        // Before the `Self` arm: a policy denial boxed into `anyhow` and
        // re-raised by a caller must keep its variant, not fall through to
        // `Other` and change the error a guest sees.
        let err = match err.downcast::<PermissionDenied>() {
            Ok(denied) => return denied.into(),
            Err(err) => err,
        };
        let err = match err.downcast::<SessionError>() {
            Ok(session) => return session.into(),
            Err(err) => err,
        };
        match err.downcast::<Self>() {
            Ok(script) => script,
            Err(err) => Self::Other(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_policy_denial_boxed_into_anyhow_keeps_its_variant() {
        assert_eq!(
            ScriptError::from(anyhow::Error::new(PermissionDenied(HostApi::Physics))),
            ScriptError::Permission(HostApi::Physics),
            "a denial that reaches a guest as `other` is a different WIT error \
             than the one raised"
        );
        assert_eq!(
            ScriptError::from(anyhow::Error::new(SessionError::NotOwner)),
            ScriptError::NotOwner,
        );
    }

    #[test]
    fn a_quota_error_keeps_the_resource_it_names() {
        assert_eq!(
            ScriptError::from(QuotaError::Stock(Stock::Prims)),
            ScriptError::QuotaStock(Stock::Prims)
        );
        assert_eq!(
            ScriptError::from(QuotaError::Flow(Flow::Emit)),
            ScriptError::QuotaFlow(Flow::Emit)
        );
    }
}
