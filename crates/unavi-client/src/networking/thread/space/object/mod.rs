//! Dynamic object networking - real-time physics state streaming.object
//!
//! Objects are owned by a single agent at a time. The owner streams
//! physics updates to all peers, who locally simulate and correct
//! based on received authoritative state.

// TODO: Implement object inbound/outbound handlers.
// pub mod inbound;
// pub mod outbound;
