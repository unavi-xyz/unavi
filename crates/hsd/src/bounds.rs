//! Largest a single stored value may be.
//!
//! Each bounds one write, not what a document or a peer may accumulate, so
//! nothing here is charged against a budget. A value over its bound is refused
//! at the point it would be stored or synced.

const KB: usize = 1024;
const MB: usize = 1024 * KB;

/// Largest payload a single `emit` may carry, fanned out to every receptor.
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 64 * KB;

/// Largest string written into a synced document (names, relationship keys).
pub const MAX_NAME_BYTES: usize = KB;

/// Largest string a single text prim may carry.
///
/// The renderer bounds a string again at layout time as a backstop for
/// documents that arrived over the network; this bound is the cheaper one,
/// stopping the bytes being stored and synced at all.
pub const MAX_TEXT_BYTES: usize = 4 * KB;

/// Largest vertex/index stream a single mesh write may upload.
pub const MAX_MESH_ELEMENTS: usize = 4 * MB;
