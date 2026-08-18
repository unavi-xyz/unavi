use unavi_policy::error::PolicyError;
use unavi_quota::QuotaError;

/// Host-side canonical error, mirroring `wired:error/types.error`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptError {
    Other(String),
    Quota(String),
    Permission(String),
    Firewall(String),
}

impl ScriptError {
    pub fn permission(detail: impl Into<String>) -> Self {
        Self::Permission(detail.into())
    }

    pub fn firewall(detail: impl Into<String>) -> Self {
        Self::Firewall(detail.into())
    }

    pub fn other(detail: impl Into<String>) -> Self {
        Self::Other(detail.into())
    }
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(s) => write!(f, "{s}"),
            Self::Quota(s) => write!(f, "quota exceeded: {s}"),
            Self::Permission(s) => write!(f, "permission denied: {s}"),
            Self::Firewall(s) => write!(f, "firewall blocked: {s}"),
        }
    }
}

impl std::error::Error for ScriptError {}

impl From<QuotaError> for ScriptError {
    fn from(err: QuotaError) -> Self {
        let detail = match err {
            QuotaError::Stock(s) => format!("{s:?}"),
            QuotaError::Flow(f) => format!("{f:?}"),
        };
        Self::Quota(detail)
    }
}

impl From<PolicyError> for ScriptError {
    fn from(err: PolicyError) -> Self {
        match err {
            PolicyError::Permission(detail) => Self::Permission(detail),
            PolicyError::Firewall(detail) => Self::Firewall(detail),
        }
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
    use super::*;

    #[test]
    fn a_policy_denial_boxed_into_anyhow_keeps_its_variant() {
        for (policy, expected) in [
            (
                PolicyError::Firewall("SceneWrite".into()),
                ScriptError::Firewall("SceneWrite".into()),
            ),
            (
                PolicyError::Permission("Physics".into()),
                ScriptError::Permission("Physics".into()),
            ),
        ] {
            let script = ScriptError::from(anyhow::Error::new(policy));
            assert_eq!(
                script, expected,
                "a denial that reaches a guest as `other` is a different WIT \
                 error than the one raised"
            );
        }
    }
}
