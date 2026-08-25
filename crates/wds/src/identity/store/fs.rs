#[cfg(unix)] use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
};

use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

const KEY_FILE: &str = "key.pem";

pub fn load_or_create(dir: &Path) -> anyhow::Result<P256KeyPair> {
    let path = dir.join(KEY_FILE);

    if path.exists() {
        let pem = Zeroizing::new(std::fs::read_to_string(&path)?);
        return P256KeyPair::from_pkcs8_pem(pem.as_str());
    }

    std::fs::create_dir_all(dir)?;
    let pair = P256KeyPair::generate();
    write_key(&path, &pair.to_pkcs8_pem()?)?;
    Ok(pair)
}

fn write_key(path: &Path, pem: &Zeroizing<String>) -> anyhow::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    options.mode(0o600);

    options.open(path)?.write_all(pem.as_bytes())?;
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
    fn a_missing_directory_is_created() {
        let dir = tempdir().expect("temp dir");
        let nested = dir.path().join("wds").join("identity");

        load_or_create(&nested).expect("create key");

        assert!(nested.join(KEY_FILE).exists());
    }
}
