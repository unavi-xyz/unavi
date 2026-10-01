//! Who may do what: how far each peer is trusted, which host APIs a document
//! may call, and the quotas that bound what it spends.
//!
//! Which peer authored a document is not decided here; the caller resolves it
//! from replicated state and asks.

pub mod ledger;
pub mod permissions;
pub mod quota;
pub mod trust;

pub use ledger::Policy;
