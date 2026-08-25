use std::path::PathBuf;

use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};

#[cfg(not(target_family = "wasm"))] pub mod fs;
#[cfg(target_family = "wasm")] pub mod web;

/// Where a node keeps the one secret it persists.
pub enum KeyStorage {
    /// Generated per process. Several clients can then share a machine without
    /// contending for one key, and a test leaves nothing behind.
    Ephemeral,
    /// A directory holding a PEM file. Not available on wasm.
    Path(PathBuf),
    /// Browser local storage. Only available on wasm.
    Browser,
}

/// Loads the key from `storage`, generating and saving one if absent.
///
/// A key that is present but unreadable is left in place and the process runs
/// under a generated one: rewriting it would destroy an identity its owner may
/// still be able to recover, and failing outright would leave the node with no
/// store at all.
pub fn load_or_create(storage: &KeyStorage) -> anyhow::Result<P256KeyPair> {
    match storage {
        KeyStorage::Ephemeral => Ok(P256KeyPair::generate()),
        #[cfg(not(target_family = "wasm"))]
        KeyStorage::Path(dir) => fs::load_or_create(dir),
        #[cfg(target_family = "wasm")]
        KeyStorage::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
        #[cfg(target_family = "wasm")]
        KeyStorage::Browser => web::load_or_create(),
        #[cfg(not(target_family = "wasm"))]
        KeyStorage::Browser => anyhow::bail!("browser storage is only supported on wasm"),
    }
}
