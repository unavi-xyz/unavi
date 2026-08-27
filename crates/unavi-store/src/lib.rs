//! This node's local data. A blob store, a document store, and the durable
//! key/value state that survives a restart.

pub mod cache;
pub mod local;
pub mod namespace;
pub mod store;
