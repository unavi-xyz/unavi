const KB: usize = 1024;
const MB: usize = 1024 * KB;

/// Largest payload a single `emit` may carry, fanned out to every receptor.
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 64 * KB;

/// Largest string written into a synced document (names, relationship keys).
pub const MAX_NAME_BYTES: usize = KB;

/// Largest string a single text prim may carry.
///
/// Enforced when the string is stored or synced. Layout re-checks
/// network-delivered text as a backstop.
pub const MAX_TEXT_BYTES: usize = 4 * KB;

/// Largest vertex/index stream a single mesh write may upload, in bytes.
pub const MAX_MESH_STREAM_BYTES: usize = 4 * MB;

/// Largest value a single document entry may hold. A mesh carries its streams
/// in one entry, so this sits several streams above [`MAX_MESH_STREAM_BYTES`].
pub const MAX_ENTRY_BYTES: usize = 32 * MB;

/// Largest `.hsdz` package a loader reads.
pub const MAX_PACKAGE_BYTES: usize = 256 * MB;

/// Most sub-documents one `.hsdz` package may carry. Each mints a namespace
/// when the package is instanced.
pub const MAX_PACKAGE_DOCUMENTS: usize = 256;
