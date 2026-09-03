//! Local state that outlives the process, addressed by string key.
//!
//! A key names the same value on every target. On native that is a file
//! beneath a root directory, on wasm an item in browser local storage keyed
//! by the same root. Keys may nest with `/`.

use std::{
    collections::HashMap,
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Arc,
        Mutex,
    },
};

#[cfg(not(target_family = "wasm"))] pub mod fs;
pub mod mem;
#[cfg(target_family = "wasm")] pub mod web;

/// The in-memory store backing [`LocalStorage::InMemory`].
type Map = HashMap<String, Vec<u8>>;

#[derive(Clone, Debug)]
pub enum LocalStorage {
    InMemory(Arc<Mutex<Map>>),
    Path(PathBuf),
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::InMemory(Arc::default())
    }
}

impl LocalStorage {
    /// The root backing this storage, or `None` when it is only in memory.
    #[must_use]
    pub fn dir(&self) -> Option<&Path> {
        match self {
            Self::Path(dir) => Some(dir),
            Self::InMemory(_) => None,
        }
    }

    /// The value recorded at `key`.
    pub fn read(&self, key: &str) -> anyhow::Result<Option<String>> {
        validate_key(key)?;
        match self {
            Self::InMemory(map) => mem::read(map, key),
            Self::Path(dir) => cfg_select! {
                target_family = "wasm" => web::read(dir, key),
                _ => fs::read(dir, key),
            },
        }
    }

    /// Records `value` at `key`, replacing whatever is there already.
    pub fn write(&self, key: &str, value: &str) -> anyhow::Result<()> {
        self.write_bytes(key, value.as_bytes())
    }

    /// Records `value` at `key` only if no value sits there yet.
    ///
    /// A racing writer gets an `Err` rather than silently replacing what the
    /// winner wrote.
    pub fn create(&self, key: &str, value: &str) -> anyhow::Result<()> {
        validate_key(key)?;
        match self {
            Self::InMemory(map) => mem::create(map, key, value.as_bytes()),
            Self::Path(dir) => cfg_select! {
                target_family = "wasm" => web::create(dir, key, value),
                _ => fs::create(dir, key, value.as_bytes()),
            },
        }
    }

    /// The raw bytes recorded at `key`. See [`Self::read`] for the shape of
    /// the result.
    pub fn read_bytes(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        validate_key(key)?;
        match self {
            Self::InMemory(map) => mem::read_bytes(map, key),
            Self::Path(dir) => cfg_select! {
                target_family = "wasm" => web::read_bytes(dir, key),
                _ => fs::read_bytes(dir, key),
            },
        }
    }

    /// Records `bytes` at `key`, replacing whatever is there already.
    pub fn write_bytes(&self, key: &str, value: &[u8]) -> anyhow::Result<()> {
        validate_key(key)?;
        match self {
            Self::InMemory(map) => mem::write_bytes(map, key, value),
            Self::Path(dir) => cfg_select! {
                target_family = "wasm" => web::write_bytes(dir, key, value),
                _ => fs::write_bytes(dir, key, value),
            },
        }
    }
}

/// A key is a `/`-separated path beneath the storage root. Anything that could
/// name a path outside it is refused rather than silently escaped.
fn validate_key(key: &str) -> anyhow::Result<()> {
    if key.is_empty()
        || key.starts_with('/')
        || key
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        anyhow::bail!("key {key:?} would escape the storage directory");
    }
    Ok(())
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn a_path_round_trips_a_value_by_key() {
        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());

        assert_eq!(storage.read("root-doc").expect("read missing"), None);

        storage.write("root-doc", "written").expect("write");

        assert_eq!(
            storage.read("root-doc").expect("read").as_deref(),
            Some("written")
        );
    }

    #[test]
    fn a_nested_key_creates_its_directories() {
        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());

        storage
            .write("registry/views/recent", "value")
            .expect("write");

        assert_eq!(
            storage
                .read("registry/views/recent")
                .expect("read")
                .as_deref(),
            Some("value")
        );
        assert_eq!(
            storage.read("registry/views/featured").expect("read"),
            None,
            "keys sharing a prefix name separate values"
        );
    }

    #[test]
    fn in_memory_keeps_state_within_a_process() {
        let storage = LocalStorage::default();

        storage.write("key", "value").expect("write");

        assert_eq!(
            storage.read("key").expect("read").as_deref(),
            Some("value"),
            "an in-memory read must answer what the same process wrote"
        );

        storage
            .create("key", "clobber")
            .expect_err("an in-memory create must refuse an existing key");

        let fresh = LocalStorage::default();
        assert_eq!(
            fresh.read("key").expect("read"),
            None,
            "a fresh in-memory storage starts empty"
        );
    }

    #[test]
    fn a_missing_value_is_absence_not_an_error() {
        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());

        assert_eq!(storage.read("absent").expect("read"), None);
    }

    #[test]
    fn an_unreadable_value_is_an_error_not_absence() {
        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());
        std::fs::write(dir.path().join("broken"), [0xFF, 0xFE]).expect("write");

        assert!(
            storage.read("broken").is_err(),
            "a value that cannot be decoded as UTF-8 must not read as absent"
        );
        assert_eq!(
            storage.read_bytes("broken").expect("raw read").as_deref(),
            Some(&[0xFF, 0xFE][..]),
            "the bytes API reports the damage instead of hiding it"
        );
    }

    #[test]
    fn keys_cannot_escape_the_storage_dir() {
        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());

        for key in ["", "/absolute", ".", "..", "../x", "a/../b", "a//b"] {
            assert!(storage.read(key).is_err(), "read must refuse {key:?}");
            assert!(
                storage.write(key, "x").is_err(),
                "write must refuse {key:?}"
            );
        }
    }

    #[test]
    fn write_replaces_a_value_atomically() {
        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());

        storage.write("key", "first").expect("write");
        storage.write("key", "second").expect("replace");

        assert_eq!(
            storage.read("key").expect("read").as_deref(),
            Some("second")
        );

        let entries = std::fs::read_dir(dir.path())
            .expect("read dir")
            .collect::<Result<Vec<_>, _>>()
            .expect("entries");
        assert_eq!(
            entries.len(),
            1,
            "a replace must leave just the value behind, not its temporaries"
        );
    }

    #[test]
    fn create_refuses_to_overwrite() {
        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());

        storage.write("key", "keep").expect("write");
        storage
            .create("key", "clobber")
            .expect_err("clobber refused");
        assert_eq!(storage.read("key").expect("read").as_deref(), Some("keep"));

        storage.create("fresh", "new").expect("create");
        assert_eq!(storage.read("fresh").expect("read").as_deref(), Some("new"));
    }

    #[test]
    fn bytes_round_trip() {
        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());

        storage
            .write_bytes("raw", &[0x00, 0x7F, 0xFF])
            .expect("write");

        assert_eq!(
            storage.read_bytes("raw").expect("read").as_deref(),
            Some(&[0x00, 0x7F, 0xFF][..])
        );
    }

    #[cfg(unix)]
    #[test]
    fn files_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().expect("temp dir");
        let storage = LocalStorage::Path(dir.path().to_path_buf());

        storage.write("written", "value").expect("write");
        storage.create("created", "value").expect("create");

        for file in ["written", "created"] {
            let meta = std::fs::metadata(dir.path().join(file)).expect("metadata");
            assert_eq!(meta.permissions().mode() & 0o777, 0o600, "{file}");
        }
    }

    #[test]
    fn in_memory_write_round_trips_bytes() {
        let storage = LocalStorage::default();

        storage
            .write_bytes("raw", &[0x00, 0x7F, 0xFF])
            .expect("write");

        assert_eq!(
            storage.read_bytes("raw").expect("read").as_deref(),
            Some(&[0x00, 0x7F, 0xFF][..])
        );
    }
}
