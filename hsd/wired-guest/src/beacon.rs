//! The beacon-to-gate contract: a beacon emits its space's id on
//! [`CHANNEL`]; a gate nearby listens on the same channel and opens to it.
//! One typed constant so the two scripts cannot drift by a typo.

/// Global event channel a beacon emits its 32-byte space id on.
pub const CHANNEL: &str = "unavi:beacon/id";
