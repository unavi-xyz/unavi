use iroh::SecretKey;
use unavi_store::local::Storage;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

pub struct Keys {
    pub identity: P256KeyPair,
    pub endpoint: SecretKey,
}

const IDENTITY_ITEM: &str = "key.pem";
const ENDPOINT_ITEM: &str = "endpoint.key";

pub fn load(storage: &Storage) -> anyhow::Result<Keys> {
    match storage {
        Storage::Ephemeral => Ok(Keys {
            identity: P256KeyPair::generate(),
            endpoint: SecretKey::generate(),
        }),
        Storage::Path(_) | Storage::Browser => Ok(Keys {
            identity: identity_key(storage)?,
            endpoint: endpoint_key(storage)?,
        }),
    }
}

/// A stored key that cannot be read is an error. Replacing it would discard
/// the identity, which nothing can recover.
fn identity_key(storage: &Storage) -> anyhow::Result<P256KeyPair> {
    if let Some(pem) = storage.read(IDENTITY_ITEM)? {
        return P256KeyPair::from_pkcs8_pem(Zeroizing::new(pem).as_str());
    }

    let pair = P256KeyPair::generate();
    storage.create(IDENTITY_ITEM, pair.to_pkcs8_pem()?.as_str())?;
    Ok(pair)
}

/// A stored key that is missing, short, or unreadable is replaced with a fresh
/// one. Losing it costs a new `EndpointId` and author id, nothing more.
fn endpoint_key(storage: &Storage) -> anyhow::Result<SecretKey> {
    match storage.read_bytes(ENDPOINT_ITEM) {
        Ok(Some(bytes)) if bytes.len() == 32 => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok(SecretKey::from_bytes(&key))
        }
        _ => {
            let key = SecretKey::generate();
            storage.write_bytes(ENDPOINT_ITEM, &key.to_bytes())?;
            Ok(key)
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use xdid::methods::key::keys::PublicKey;

    use super::*;

    fn path_storage() -> (tempfile::TempDir, Storage) {
        let dir = tempdir().expect("temp dir");
        let storage = Storage::Path(dir.path().to_path_buf());
        (dir, storage)
    }

    #[test]
    fn keys_survive_a_reload() {
        let (_dir, storage) = path_storage();

        let created = load(&storage).expect("create keys");
        let reloaded = load(&storage).expect("load keys");

        assert_eq!(
            created.identity.public().to_did(),
            reloaded.identity.public().to_did()
        );
        assert_eq!(created.endpoint.to_bytes(), reloaded.endpoint.to_bytes());
    }

    #[test]
    fn ephemeral_mints_fresh_keys_every_load() {
        let storage = Storage::Ephemeral;

        let first = load(&storage).expect("first load");
        let second = load(&storage).expect("second load");

        assert_ne!(
            first.identity.public().to_did(),
            second.identity.public().to_did()
        );
        assert_ne!(first.endpoint.to_bytes(), second.endpoint.to_bytes());
    }

    #[cfg(unix)]
    #[test]
    fn key_files_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let (dir, storage) = path_storage();
        load(&storage).expect("create keys");

        for file in [IDENTITY_ITEM, ENDPOINT_ITEM] {
            let meta = std::fs::metadata(dir.path().join(file)).expect("metadata");
            assert_eq!(meta.permissions().mode() & 0o777, 0o600, "{file}");
        }
    }

    #[test]
    fn a_short_endpoint_key_is_replaced() {
        let (dir, storage) = path_storage();
        std::fs::write(dir.path().join(ENDPOINT_ITEM), b"short").expect("write");

        let key = endpoint_key(&storage).expect("replace key");

        assert_eq!(
            key.to_bytes(),
            endpoint_key(&storage).expect("load key").to_bytes(),
            "a partial write is rewritten rather than failing every later load"
        );
    }

    #[test]
    fn an_unreadable_identity_key_is_preserved() {
        let (dir, storage) = path_storage();
        std::fs::write(dir.path().join(IDENTITY_ITEM), b"not a pem").expect("write");

        assert!(
            load(&storage).is_err(),
            "an identity is irreplaceable, so an unreadable key must fail the load"
        );
        assert_eq!(
            std::fs::read(dir.path().join(IDENTITY_ITEM)).expect("read back"),
            b"not a pem",
            "the key its owner may still recover must be left on disk"
        );
    }
}
