//! A registry: a node that lists public namespaces and who is in them.
//!
//! A listing ([`claim::Submission`]) is durable and keyed by namespace. Only a
//! DID that authors the namespace may list it, and once listed, only that DID
//! may replace or retract the listing until it expires. Presence
//! ([`claim::Presence`]) is held in memory only. Clients read curated views
//! ([`views::ViewIds`]) by syncing them as ordinary documents.
//!
//! The `server` feature runs one; the `client` feature follows them.

pub mod claim;
pub mod error;
pub mod rpc;
pub mod views;

#[cfg(feature = "client")] pub mod client;
#[cfg(feature = "server")] pub mod server;
