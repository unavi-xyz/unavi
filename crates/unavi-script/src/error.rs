use unavi_quota::QuotaError;

/// Host-side canonical error, mirroring `wired:error/types.error`. Each fallible
/// host binding lowers it via the single `From` impl in
/// [`crate::runtime::native::wired::error`].
#[derive(Debug, Clone)]
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

impl From<anyhow::Error> for ScriptError {
    fn from(err: anyhow::Error) -> Self {
        let err = match err.downcast::<QuotaError>() {
            Ok(quota) => return quota.into(),
            Err(err) => err,
        };
        match err.downcast::<Self>() {
            Ok(script) => script,
            Err(err) => Self::Other(err.to_string()),
        }
    }
}
