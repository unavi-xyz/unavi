//! The pin/hold/session state replicated between peers in a space.
//!
//! A pin says a peer is present with a document it authors, and a hold gives a
//! peer simulation authority over a document's bodies. A session layer is one
//! writer's signed opinions on a document's prims, stacked above the document
//! and held until its writer and every peer carrying it are gone. [`record`]
//! holds the author's signed word on whether a document is pinned.
//! [`store::Replicas`] keeps the state and its rules, [`guards`] ties each
//! piece to an entity, [`session`] stacks the layers, and [`wire`] carries it
//! all over the state stream.

pub mod clock;
pub mod frames;
pub mod guards;
pub mod message;
pub mod record;
pub mod request;
pub mod session;
pub mod snapshot;
pub mod store;
pub(crate) mod wire;

pub use guards::LocalReplica;
pub use store::Replicas;
