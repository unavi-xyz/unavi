use std::path::PathBuf;

use iroh::SecretKey;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};

#[cfg(not(target_family = "wasm"))] pub mod fs;
#[cfg(target_family = "wasm")] pub mod web;

/// Where a node keeps the local state that must outlive a process: its user
/// key and its endpoint key.
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
        KeyStorage::Path(_dir) => {
            cfg_select! {
                target_family = "wasm" => anyhow::bail!("file storage is not supported on wasm"),
                _ => fs::load_or_create(_dir),
            }
        }
        KeyStorage::Browser => {
            cfg_select! {
                target_family = "wasm" => web::load_or_create(),
                _ => anyhow::bail!("browser storage is only supported on wasm"),
            }
        }
    }
}

/// Loads this device's endpoint key from `storage`, generating and saving one
/// if absent.
///
/// An unreadable key is replaced rather than preserved, which is the opposite
/// of [`load_or_create`]'s rule: an identity key is irreplaceable, while a lost
/// endpoint key costs only a new `EndpointId` and author id.
pub fn load_or_create_endpoint(storage: &KeyStorage) -> anyhow::Result<SecretKey> {
    match storage {
        KeyStorage::Ephemeral => Ok(SecretKey::generate()),
        KeyStorage::Path(_dir) => {
            cfg_select! {
                target_family = "wasm" => anyhow::bail!("file storage is not supported on wasm"),
                _ => fs::load_or_create_endpoint(_dir),
            }
        }
        KeyStorage::Browser => {
            cfg_select! {
                target_family = "wasm" => web::load_or_create_endpoint(),
                _ => anyhow::bail!("browser storage is only supported on wasm"),
            }
        }
    }
}
