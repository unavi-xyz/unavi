use iroh::SecretKey;
use unavi_store::local::Storage;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};

#[cfg(not(target_family = "wasm"))] pub mod fs;
#[cfg(target_family = "wasm")] pub mod web;

/// Loads the key from `storage`, generating and saving one if absent.
///
/// A key that is present but unreadable is left in place and the process runs
/// under a generated one: rewriting it would destroy an identity its owner may
/// still be able to recover, and failing outright would leave the node with no
/// store at all.
pub fn load_or_create(storage: &Storage) -> anyhow::Result<P256KeyPair> {
    match storage {
        Storage::Ephemeral => Ok(P256KeyPair::generate()),
        Storage::Path(_dir) => {
            cfg_select! {
                target_family = "wasm" => anyhow::bail!("file storage is not supported on wasm"),
                _ => fs::load_or_create(_dir),
            }
        }
        Storage::Browser => {
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
pub fn load_or_create_endpoint(storage: &Storage) -> anyhow::Result<SecretKey> {
    match storage {
        Storage::Ephemeral => Ok(SecretKey::generate()),
        Storage::Path(_dir) => {
            cfg_select! {
                target_family = "wasm" => anyhow::bail!("file storage is not supported on wasm"),
                _ => fs::load_or_create_endpoint(_dir),
            }
        }
        Storage::Browser => {
            cfg_select! {
                target_family = "wasm" => web::load_or_create_endpoint(),
                _ => anyhow::bail!("browser storage is only supported on wasm"),
            }
        }
    }
}
