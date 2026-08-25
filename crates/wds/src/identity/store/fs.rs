#[cfg(unix)] use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::OpenOptions,
    io::Write,
    path::{
        Path,
        PathBuf,
    },
    str::FromStr,
};

use iroh_docs::NamespaceId;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

use crate::identity::store::DeviceSeed;

const KEY_FILE: &str = "key.pem";
const SEED_FILE: &str = "device.seed";
const NAMESPACE_DIR: &str = "namespaces";

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

/// Deleting [`SEED_FILE`] is how a device rotates: the next load writes a new
/// seed, and with it a new endpoint id and author id.
pub fn load_or_create_seed(dir: &Path) -> anyhow::Result<DeviceSeed> {
    let path = dir.join(SEED_FILE);

    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice())
    {
        return Ok(DeviceSeed::from_bytes(bytes));
    }

    std::fs::create_dir_all(dir)?;
    let seed = DeviceSeed::generate();
    // Truncating rather than `create_new`: a seed too short to parse is a
    // partial write, and rewriting it costs only a new endpoint id.
    let _ = std::fs::remove_file(&path);
    write_secret(&path, seed.as_bytes())?;
    Ok(seed)
}

/// A label is a document's name, so it is also its file's name. Labels come
/// from [`crate::identity::labels`] and never from a peer, so the only
/// separator to fold away is the one they use themselves.
fn namespace_path(dir: &Path, label: &str) -> PathBuf {
    dir.join(NAMESPACE_DIR).join(label.replace('/', "_"))
}

pub fn load_namespace(dir: &Path, label: &str) -> anyhow::Result<Option<NamespaceId>> {
    let Ok(text) = std::fs::read_to_string(namespace_path(dir, label)) else {
        return Ok(None);
    };
    Ok(NamespaceId::from_str(text.trim()).ok())
}

pub fn save_namespace(dir: &Path, label: &str, ns: NamespaceId) -> anyhow::Result<()> {
    let path = namespace_path(dir, label);
    std::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| anyhow::anyhow!("namespace path has no parent"))?,
    )?;
    std::fs::write(path, ns.to_string())?;
    Ok(())
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
    fn a_seed_survives_a_reload_and_rotates_when_deleted() {
        let dir = tempdir().expect("temp dir");

        let created = load_or_create_seed(dir.path()).expect("create seed");
        let reloaded = load_or_create_seed(dir.path()).expect("load seed");
        assert_eq!(created.as_bytes(), reloaded.as_bytes());

        std::fs::remove_file(dir.path().join(SEED_FILE)).expect("rotate");
        let rotated = load_or_create_seed(dir.path()).expect("create seed");

        assert_ne!(
            created.as_bytes(),
            rotated.as_bytes(),
            "deleting the seed is how a device takes a new endpoint id"
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
    fn seed_file_is_owner_only() {
        let dir = tempdir().expect("temp dir");
        load_or_create_seed(dir.path()).expect("create seed");

        let meta = std::fs::metadata(dir.path().join(SEED_FILE)).expect("metadata");

        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn a_truncated_seed_is_replaced() {
        let dir = tempdir().expect("temp dir");
        std::fs::create_dir_all(dir.path()).expect("mkdir");
        std::fs::write(dir.path().join(SEED_FILE), b"short").expect("write");

        let seed = load_or_create_seed(dir.path()).expect("create seed");

        assert_eq!(
            seed.as_bytes(),
            load_or_create_seed(dir.path())
                .expect("load seed")
                .as_bytes(),
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
