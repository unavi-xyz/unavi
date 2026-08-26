use iroh::SecretKey;
use unavi_store::local::Storage;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};

#[cfg(not(target_family = "wasm"))] mod fs;
#[cfg(target_family = "wasm")] mod web;

pub struct Keys {
    pub identity: P256KeyPair,
    pub endpoint: SecretKey,
}

pub fn load(storage: &Storage) -> anyhow::Result<Keys> {
    match storage {
        Storage::Ephemeral => Ok(Keys {
            identity: P256KeyPair::generate(),
            endpoint: SecretKey::generate(),
        }),
        Storage::Path(dir) => {
            cfg_select! {
                target_family = "wasm" => {
                    anyhow::bail!("file storage is not supported on wasm: {}", dir.display())
                }
                _ => fs::load(dir),
            }
        }
        Storage::Browser => {
            cfg_select! {
                target_family = "wasm" => web::load(),
                _ => anyhow::bail!("browser storage is only supported on wasm"),
            }
        }
    }
}
