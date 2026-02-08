//! Dynamic object networking - real-time physics state streaming.
//!
//! Objects are owned by a single agent at a time. The owner streams
//! physics updates to all peers, who locally simulate and correct
//! based on received authoritative state.
//!
//! ## Stream Architecture
//!
//! Each object gets its own dedicated bistream within the shared connection:
//! - Stream init: `StreamInit::Object { object_id }` identifies the stream
//! - I-frames: Sent reliably on the stream (`object_id` known from init)
//! - P-frames: Sent as datagrams, tagged with object ID for routing

pub mod inbound;
pub mod outbound;
