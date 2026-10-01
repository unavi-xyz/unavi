//! The pin/hold/session state replicated between peers in a space.
//!
//! A pin says a peer is present with a document it authors; a hold gives a
//! peer simulation authority over a document's bodies; session cells are
//! opinions on a document's prims that last as long as the session does.
//! [`store::Replicas`] keeps the record and its rules, [`guards`] ties each
//! piece to an entity, and [`wire`] carries it over the state stream.

pub mod cell;
pub mod clock;
pub mod guards;
pub mod message;
pub mod snapshot;
pub mod store;
pub(crate) mod wire;

pub use guards::LocalReplica;
pub use store::Replicas;
