//! The pin/hold/session state replicated between peers in a space.
//!
//! A pin says a peer is present with a document it authors, and a hold gives a
//! peer simulation authority over a document's bodies. A session layer is one
//! present peer's opinions on a document's prims, stacked above the document
//! and gone when the peer is. [`store::Replicas`] keeps the record and its
//! rules, [`guards`] ties each piece to an entity, [`session`] stacks the
//! layers, and [`wire`] carries it all over the state stream.

pub mod clock;
pub mod guards;
pub mod message;
pub mod session;
pub mod snapshot;
pub mod store;
pub(crate) mod wire;

pub use guards::LocalReplica;
pub use store::Replicas;
