/// A denial by one of the policy layers.
///
/// Carries what was denied rather than a rendered sentence, so a caller can
/// report which permission was missing or why the write was out of reach.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("out of reach: {0}")]
    Reach(String),
}
