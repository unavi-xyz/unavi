use std::path::PathBuf;

use iroh_docs::NamespaceId;
use rand::RngCore;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

#[cfg(not(target_family = "wasm"))] pub mod fs;
#[cfg(target_family = "wasm")] pub mod web;

/// The secret belonging to one device rather than to the person.
///
/// Backs the endpoint key and the docs author, neither of which may be derived
/// from the identity key: an `EndpointId` is what discovery maps to addresses,
/// so two of a person's devices sharing one would publish a merged address set
/// under a single identifier.
///
/// Rotating is deleting it. The next load generates a fresh one, giving the
/// device a new endpoint and a new author while the identity key and the DID
/// stay exactly as they were.
#[derive(Clone)]
pub struct DeviceSeed(Zeroizing<[u8; 32]>);

impl DeviceSeed {
    #[must_use]
    pub fn generate() -> Self {
        let mut bytes = Zeroizing::new([0u8; 32]);
        rand::rng().fill_bytes(bytes.as_mut_slice());
        Self(bytes)
    }

    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Where a node keeps the local state that must outlive a process: its identity
/// key, its device seed, and the ids of the documents it minted.
#[derive(Clone)]
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

/// The namespace this node minted for `label`, if it has minted one.
///
/// A well-known document is named by a pointer rather than by derivation: a
/// namespace id computed from a secret is one nobody else can compute, and a
/// document exists to be read by someone.
pub fn load_namespace(storage: &KeyStorage, label: &str) -> anyhow::Result<Option<NamespaceId>> {
    match storage {
        KeyStorage::Ephemeral => Ok(None),
        #[cfg(not(target_family = "wasm"))]
        KeyStorage::Path(dir) => fs::load_namespace(dir, label),
        #[cfg(target_family = "wasm")]
        KeyStorage::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
        #[cfg(target_family = "wasm")]
        KeyStorage::Browser => web::load_namespace(label),
        #[cfg(not(target_family = "wasm"))]
        KeyStorage::Browser => anyhow::bail!("browser storage is only supported on wasm"),
    }
}

/// Records `ns` as this node's namespace for `label`.
///
/// [`KeyStorage::Ephemeral`] keeps nothing, so a process using it mints a fresh
/// namespace for every label on every run.
pub fn save_namespace(storage: &KeyStorage, label: &str, ns: NamespaceId) -> anyhow::Result<()> {
    match storage {
        KeyStorage::Ephemeral => Ok(()),
        #[cfg(not(target_family = "wasm"))]
        KeyStorage::Path(dir) => fs::save_namespace(dir, label, ns),
        #[cfg(target_family = "wasm")]
        KeyStorage::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
        #[cfg(target_family = "wasm")]
        KeyStorage::Browser => web::save_namespace(label, ns),
        #[cfg(not(target_family = "wasm"))]
        KeyStorage::Browser => anyhow::bail!("browser storage is only supported on wasm"),
    }
}

/// Loads this device's seed from `storage`, generating and saving one if
/// absent.
///
/// An unreadable seed is replaced rather than preserved, which is the opposite
/// of [`load_or_create`]'s rule: an identity key is irreplaceable, while a lost
/// seed costs only a new endpoint id.
pub fn load_or_create_seed(storage: &KeyStorage) -> anyhow::Result<DeviceSeed> {
    match storage {
        KeyStorage::Ephemeral => Ok(DeviceSeed::generate()),
        #[cfg(not(target_family = "wasm"))]
        KeyStorage::Path(dir) => fs::load_or_create_seed(dir),
        #[cfg(target_family = "wasm")]
        KeyStorage::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
        #[cfg(target_family = "wasm")]
        KeyStorage::Browser => web::load_or_create_seed(),
        #[cfg(not(target_family = "wasm"))]
        KeyStorage::Browser => anyhow::bail!("browser storage is only supported on wasm"),
    }
}
