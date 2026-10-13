//! Trust, permissions and quota, and the rules that map one to the others.
//!
//! # Concepts
//!
//! `trust` ranks peers, `permissions` names the host APIs a document may call,
//! and `quota` bounds what it spends. Each is independent of the other two.
//!
//! # Rules
//!
//! `rules` maps a trust rung to the permissions and limits it is granted.
//!
//! # Ledger
//!
//! `ledger` records every document and the quota it charges. The caller
//! resolves a document's author and asks.

pub mod ledger;
pub mod permissions;
pub mod quota;
pub mod rules;
pub mod trust;

pub use ledger::Policy;
