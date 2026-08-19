use unavi_policy::error::PolicyError;
use unavi_quota::{
    Flow,
    QuotaError,
    Stock,
};
use unavi_space::state::replicas::KvError;

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
    #[error(transparent)]
    Policy(PolicyError),
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

impl From<KvError> for ScriptError {
    fn from(err: KvError) -> Self {
        match err {
            KvError::QuotaExceeded => Self::QuotaStock(Stock::KvMemory),
            KvError::NotOwner => Self::Policy(PolicyError::NotOwner),
            KvError::KeyTooLong | KvError::Other => Self::Other(err.to_string()),
        }
    }
}

impl From<PolicyError> for ScriptError {
    fn from(err: PolicyError) -> Self {
        Self::Policy(err)
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
        let err = match err.downcast::<PolicyError>() {
            Ok(policy) => return policy.into(),
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
    use unavi_policy::{
        document::ApiName,
        trust::Trust,
    };

    use super::*;

    #[test]
    fn a_policy_denial_boxed_into_anyhow_keeps_its_variant() {
        for policy in [
            PolicyError::NotCoPresent,
            PolicyError::Rung {
                required: Trust::Trusted,
                actual:   Trust::Guest,
            },
            PolicyError::Permission(ApiName::Physics),
        ] {
            assert_eq!(
                ScriptError::from(anyhow::Error::new(policy)),
                ScriptError::Policy(policy),
                "a denial that reaches a guest as `other` is a different WIT \
                 error than the one raised"
            );
        }
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
