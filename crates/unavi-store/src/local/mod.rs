//! Local state that must outlive the process, addressed by string key.
//!
//! One key names the same value on every target: a file beneath a directory on
//! native, an item in browser local storage on wasm. Keys may nest with `/`.

use std::path::{
    Path,
    PathBuf,
};

#[cfg(not(target_family = "wasm"))] pub mod fs;
#[cfg(target_family = "wasm")] pub mod web;

/// Where a node keeps its durable local state: keys, and the ids of the
/// documents they authored.
#[derive(Clone, Debug)]
pub enum Storage {
    /// Nothing is written and every read misses, so a process mints fresh
    /// state and leaves nothing behind. Several nodes can then share a machine
    /// without contending for one directory.
    Ephemeral,
    /// A directory on disk. Not available on wasm.
    Path(PathBuf),
    /// Browser local storage. Only available on wasm.
    Browser,
}

impl Storage {
    /// The directory backing this storage, or `None` when it has no
    /// filesystem behind it.
    #[must_use]
    pub fn dir(&self) -> Option<&Path> {
        match self {
            Self::Path(dir) => Some(dir),
            Self::Ephemeral | Self::Browser => None,
        }
    }

    /// The value recorded at `key`, or `None` if this storage holds none.
    ///
    /// An unreadable value is indistinguishable from an absent one: callers
    /// mint a replacement either way.
    #[must_use]
    pub fn read(&self, key: &str) -> Option<String> {
        cfg_select! {
            target_family = "wasm" => match self {
                Self::Browser => web::read(key),
                Self::Ephemeral | Self::Path(_) => None,
            },
            _ => fs::read(self.dir()?, key),
        }
    }

    pub fn write(&self, key: &str, value: &str) -> anyhow::Result<()> {
        cfg_select! {
            target_family = "wasm" => match self {
                Self::Ephemeral => Ok(()),
                Self::Browser => web::write(key, value),
                Self::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
            },
            _ => match self {
                Self::Ephemeral => Ok(()),
                Self::Path(dir) => fs::write(dir, key, value),
                Self::Browser => anyhow::bail!("browser storage is only supported on wasm"),
            },
        }
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn a_path_round_trips_a_value_by_key() {
        let dir = tempdir().expect("temp dir");
        let storage = Storage::Path(dir.path().to_path_buf());

        assert_eq!(storage.read("root-doc"), None);

        storage.write("root-doc", "written").expect("write");

        assert_eq!(storage.read("root-doc").as_deref(), Some("written"));
    }

    #[test]
    fn a_nested_key_creates_its_directories() {
        let dir = tempdir().expect("temp dir");
        let storage = Storage::Path(dir.path().to_path_buf());

        storage
            .write("registry/views/recent", "value")
            .expect("write");

        assert_eq!(
            storage.read("registry/views/recent").as_deref(),
            Some("value")
        );
        assert_eq!(
            storage.read("registry/views/featured"),
            None,
            "keys sharing a prefix name separate values"
        );
    }

    #[test]
    fn ephemeral_keeps_nothing() {
        let storage = Storage::Ephemeral;

        storage.write("key", "value").expect("write is a no-op");

        assert_eq!(storage.read("key"), None);
    }

    #[test]
    fn browser_storage_is_rejected_off_wasm() {
        assert!(Storage::Browser.write("key", "value").is_err());
        assert_eq!(Storage::Browser.read("key"), None);
    }
}
