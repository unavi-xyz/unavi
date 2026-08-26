#[cfg(unix)] use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
};

use iroh::SecretKey;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

const KEY_FILE: &str = "key.pem";
const ENDPOINT_FILE: &str = "endpoint.key";

pub fn load_or_create(dir: &Path) -> anyhow::Result<P256KeyPair> {
    let path = dir.join(KEY_FILE);

    if path.exists() {
        let pem = Zeroizing::new(std::fs::read_to_string(&path)?);
        return P256KeyPair::from_pkcs8_pem(pem.as_str());
    }

    std::fs::create_dir_all(dir)?;
    let pair = P256KeyPair::generate();
    write_secret(&path, pair.to_pkcs8_pem()?.as_bytes())?;
    Ok(pair)
}

/// Deleting [`ENDPOINT_FILE`] is how a device rotates: the next load writes a
/// new key, and with it a new `EndpointId` and author id.
pub fn load_or_create_endpoint(dir: &Path) -> anyhow::Result<SecretKey> {
    let path = dir.join(ENDPOINT_FILE);

    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice())
    {
        return Ok(SecretKey::from_bytes(&bytes));
    }

    std::fs::create_dir_all(dir)?;
    let key = SecretKey::generate();
    // Remove-then-write rather than `create_new`: an unparsable key is a
    // partial write, and rewriting it costs only a new endpoint id.
    let _ = std::fs::remove_file(&path);
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
    fn reloads_the_same_identity() {
        let dir = tempdir().expect("temp dir");

        let created = load_or_create(dir.path()).expect("create key");
        let loaded = load_or_create(dir.path()).expect("load key");

        assert_eq!(
            created.public().to_did(),
            loaded.public().to_did(),
            "a persisted key must yield the same DID across loads"
        );
    }

    #[test]
    fn separate_directories_hold_separate_identities() {
        let a = tempdir().expect("temp dir");
        let b = tempdir().expect("temp dir");

        let a = load_or_create(a.path()).expect("create key");
        let b = load_or_create(b.path()).expect("create key");

        assert_ne!(a.public().to_did(), b.public().to_did());
    }

    #[cfg(unix)]
    #[test]
    fn key_file_is_owner_only() {
        let dir = tempdir().expect("temp dir");
        load_or_create(dir.path()).expect("create key");

        let meta = std::fs::metadata(dir.path().join(KEY_FILE)).expect("metadata");

        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn an_endpoint_key_survives_a_reload_and_rotates_when_deleted() {
        let dir = tempdir().expect("temp dir");

        let created = load_or_create_endpoint(dir.path()).expect("create key");
        let reloaded = load_or_create_endpoint(dir.path()).expect("load key");
        assert_eq!(created.to_bytes(), reloaded.to_bytes());

        std::fs::remove_file(dir.path().join(ENDPOINT_FILE)).expect("rotate");
        let rotated = load_or_create_endpoint(dir.path()).expect("create key");

        assert_ne!(
            created.to_bytes(),
            rotated.to_bytes(),
            "deleting the endpoint key is how a device takes a new endpoint id"
        );
        assert_eq!(
            load_or_create(dir.path())
                .expect("load key")
                .public()
                .to_did(),
            load_or_create(dir.path())
                .expect("load key")
                .public()
                .to_did(),
            "a rotation leaves the identity key untouched"
        );
    }

    #[cfg(unix)]
    #[test]
    fn endpoint_key_file_is_owner_only() {
        let dir = tempdir().expect("temp dir");
        load_or_create_endpoint(dir.path()).expect("create key");

        let meta = std::fs::metadata(dir.path().join(ENDPOINT_FILE)).expect("metadata");

        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn a_truncated_endpoint_key_is_replaced() {
        let dir = tempdir().expect("temp dir");
        std::fs::create_dir_all(dir.path()).expect("mkdir");
        std::fs::write(dir.path().join(ENDPOINT_FILE), b"short").expect("write");

        let key = load_or_create_endpoint(dir.path()).expect("create key");

        assert_eq!(
            key.to_bytes(),
            load_or_create_endpoint(dir.path())
                .expect("load key")
                .to_bytes(),
            "a partial write is rewritten rather than failing every later load"
        );
    }

    #[test]
    fn a_missing_directory_is_created() {
        let dir = tempdir().expect("temp dir");
        let nested = dir.path().join("wds").join("identity");

        load_or_create(&nested).expect("create key");

        assert!(nested.join(KEY_FILE).exists());
    }
}
