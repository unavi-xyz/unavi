#[cfg(unix)] use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::OpenOptions,
    io::{
        ErrorKind,
        Write,
    },
    path::Path,
};

use anyhow::Context;
use iroh::SecretKey;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

use crate::identity::store::Keys;

const KEY_FILE: &str = "key.pem";
const ENDPOINT_FILE: &str = "endpoint.key";

pub fn load(dir: &Path) -> anyhow::Result<Keys> {
    std::fs::create_dir_all(dir)?;

    Ok(Keys {
        identity: identity_key(dir)?,
        endpoint: endpoint_key(dir)?,
    })
}

/// An unreadable key fails the load rather than being replaced: rewriting it
/// would destroy an identity its owner may still be able to recover.
fn identity_key(dir: &Path) -> anyhow::Result<P256KeyPair> {
    let path = dir.join(KEY_FILE);

    if path.exists() {
        let pem = Zeroizing::new(std::fs::read_to_string(&path)?);
        return P256KeyPair::from_pkcs8_pem(pem.as_str());
    }

    let pair = P256KeyPair::generate();
    write_secret(&path, pair.to_pkcs8_pem()?.as_bytes())?;
    Ok(pair)
}

/// An unreadable key is replaced, the opposite of [`identity_key`]'s rule: a
/// lost endpoint key costs only a new `EndpointId` and author id. Deleting
/// [`ENDPOINT_FILE`] is how a device rotates.
fn endpoint_key(dir: &Path) -> anyhow::Result<SecretKey> {
    let path = dir.join(ENDPOINT_FILE);

    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice())
    {
        return Ok(SecretKey::from_bytes(&bytes));
    }

    let key = SecretKey::generate();

    // `write_secret` refuses to clobber, so the unreadable file goes first.
    if let Err(err) = std::fs::remove_file(&path)
        && err.kind() != ErrorKind::NotFound
    {
        return Err(err).context("clear the unreadable endpoint key");
    }

    write_secret(&path, &key.to_bytes())?;
    Ok(key)
}

fn write_secret(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    options.mode(0o600);

    options.open(path)?.write_all(bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)] use std::os::unix::fs::PermissionsExt;

    use tempfile::tempdir;
    use xdid::methods::key::keys::PublicKey;

    use super::*;

    #[test]
    fn keys_survive_a_reload() {
        let dir = tempdir().expect("temp dir");

        let created = load(dir.path()).expect("create keys");
        let reloaded = load(dir.path()).expect("load keys");

        assert_eq!(
            created.identity.public().to_did(),
            reloaded.identity.public().to_did()
        );
        assert_eq!(created.endpoint.to_bytes(), reloaded.endpoint.to_bytes());
    }

    #[cfg(unix)]
    #[test]
    fn key_files_are_owner_only() {
        let dir = tempdir().expect("temp dir");
        load(dir.path()).expect("create keys");

        for file in [KEY_FILE, ENDPOINT_FILE] {
            let meta = std::fs::metadata(dir.path().join(file)).expect("metadata");
            assert_eq!(meta.permissions().mode() & 0o777, 0o600, "{file}");
        }
    }

    #[test]
    fn a_truncated_endpoint_key_is_replaced() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join(ENDPOINT_FILE), b"short").expect("write");

        let key = endpoint_key(dir.path()).expect("replace key");

        assert_eq!(
            key.to_bytes(),
            endpoint_key(dir.path()).expect("load key").to_bytes(),
            "a partial write is rewritten rather than failing every later load"
        );
    }

    #[test]
    fn an_unreadable_identity_key_is_preserved() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join(KEY_FILE);
        std::fs::write(&path, b"not a pem").expect("write");

        assert!(
            identity_key(dir.path()).is_err(),
            "an identity is irreplaceable, so an unreadable key must fail the load"
        );
        assert_eq!(
            std::fs::read(&path).expect("read back"),
            b"not a pem",
            "the key its owner may still recover must be left on disk"
        );
    }
}
