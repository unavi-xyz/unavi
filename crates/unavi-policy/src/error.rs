/// A denial by one of the policy layers.
///
/// Carries the name of what was denied rather than a rendered sentence, so a
/// caller can report which permission or channel was missing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("firewall blocked: {0}")]
    Firewall(String),
}
