const KB: usize = 1024;
const MB: usize = 1024 * KB;

/// Largest payload a single `emit` may carry, fanned out to every receptor.
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 64 * KB;

/// Largest string written into a synced document (names, relationship keys).
pub const MAX_NAME_BYTES: usize = KB;

/// Largest string a single text prim may carry.
///
/// Enforced when the string is stored or synced, so network-delivered text is
/// re-checked at layout time as a backstop rather than trusted here.
pub const MAX_TEXT_BYTES: usize = 4 * KB;

/// Largest vertex/index stream a single mesh write may upload.
pub const MAX_MESH_ELEMENTS: usize = 4 * MB;
