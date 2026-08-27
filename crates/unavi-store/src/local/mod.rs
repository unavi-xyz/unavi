//! Local state that must outlive the process, addressed by string key.
//!
//! One key names the same value on every target: a file beneath a directory on
//! native, an item in browser local storage on wasm. Keys may nest with `/`.
//!
//! A read reports three states, not two: `Ok(Some(value))` when a value is
//! there, `Ok(None)` when none has ever been written, and `Err` when a value
//! sits there but cannot be read. Callers that must tell a first run from a
//! broken value — an identity key that must not be replaced, a trust table
//! whose loss unblocks every ejected peer — decide on the third state
//! themselves rather than having it silently collapse into absence.
//!
//! Writes are atomic where the platform allows: `write` replaces a value by
//! renaming a fully-written temporary over it, so a crash mid-write leaves
//! whatever was there before; `create` refuses to clobber an existing value.
//! Files on native are owner-only.

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

    /// The value recorded at `key`.
    ///
    /// `Ok(None)` when this storage holds none, `Err` when a value is present
    /// but unreadable. A [`Storage::Ephemeral`] read always misses.
    pub fn read(&self, key: &str) -> anyhow::Result<Option<String>> {
        validate_key(key)?;
        cfg_select! {
            target_family = "wasm" => match self {
                Self::Ephemeral => Ok(None),
                Self::Browser => web::read(key),
                Self::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
            },
            _ => match self {
                Self::Ephemeral => Ok(None),
                Self::Browser => anyhow::bail!("browser storage is only supported on wasm"),
                Self::Path(dir) => fs::read(dir, key),
            },
        }
    }

    /// Records `value` at `key`, replacing whatever is there already.
    ///
    /// A no-op on [`Storage::Ephemeral`], which never holds anything.
    pub fn write(&self, key: &str, value: &str) -> anyhow::Result<()> {
        self.write_bytes(key, value.as_bytes())
    }

    /// Records `value` at `key` only if no value sits there yet.
    ///
    /// A racing writer therefore gets an `Err` instead of a silent clobber, so
    /// a caller can re-read instead of losing what the winner wrote.
    pub fn create(&self, key: &str, value: &str) -> anyhow::Result<()> {
        validate_key(key)?;
        cfg_select! {
            target_family = "wasm" => match self {
                Self::Ephemeral => Ok(()),
                Self::Browser => web::create(key, value),
                Self::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
            },
            _ => match self {
                Self::Ephemeral => Ok(()),
                Self::Browser => anyhow::bail!("browser storage is only supported on wasm"),
                Self::Path(dir) => fs::create(dir, key, value.as_bytes()),
            },
        }
    }

    /// The raw bytes recorded at `key`. See [`Self::read`] for the shape of
    /// the result.
    pub fn read_bytes(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        validate_key(key)?;
        cfg_select! {
            target_family = "wasm" => match self {
                Self::Ephemeral => Ok(None),
                Self::Browser => web::read_bytes(key),
                Self::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
            },
            _ => match self {
                Self::Ephemeral => Ok(None),
                Self::Browser => anyhow::bail!("browser storage is only supported on wasm"),
                Self::Path(dir) => fs::read_bytes(dir, key),
            },
        }
    }

    /// Records `bytes` at `key`, replacing whatever is there already.
    pub fn write_bytes(&self, key: &str, value: &[u8]) -> anyhow::Result<()> {
        validate_key(key)?;
        cfg_select! {
            target_family = "wasm" => match self {
                Self::Ephemeral => Ok(()),
                Self::Browser => web::write_bytes(key, value),
                Self::Path(_) => anyhow::bail!("file storage is not supported on wasm"),
            },
            _ => match self {
                Self::Ephemeral => Ok(()),
                Self::Browser => anyhow::bail!("browser storage is only supported on wasm"),
                Self::Path(dir) => fs::write_bytes(dir, key, value),
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

/// Hex-encodes `bytes`, one pair per byte and no `0x` prefix.
#[must_use]
pub fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Decodes hex, walking bytes rather than `&str` pairs so a non-ASCII
/// character can never be sliced at a char boundary. Odd length or a
/// non-hex character is an `Err`, never a panic.
pub fn decode_hex(text: &str) -> anyhow::Result<Vec<u8>> {
    const fn nibble(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }

    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        anyhow::bail!("a hex string has even length: {}", bytes.len());
    }

    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let hi = nibble(pair[0]).ok_or_else(|| anyhow::anyhow!("{text:?} is not hex"))?;
            let lo = nibble(pair[1]).ok_or_else(|| anyhow::anyhow!("{text:?} is not hex"))?;
            Ok(hi << 4 | lo)
        })
        .collect()
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn a_path_round_trips_a_value_by_key() {
        let dir = tempdir().expect("temp dir");
        let storage = Storage::Path(dir.path().to_path_buf());

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
        let storage = Storage::Path(dir.path().to_path_buf());

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
    fn ephemeral_keeps_nothing() {
        let storage = Storage::Ephemeral;

        storage.write("key", "value").expect("write is a no-op");

        assert_eq!(storage.read("key").expect("read"), None);
    }

    #[test]
    fn browser_storage_is_rejected_off_wasm() {
        assert!(Storage::Browser.write("key", "value").is_err());
        assert!(Storage::Browser.read("key").is_err());
    }

    #[test]
    fn a_missing_value_is_absence_not_an_error() {
        let dir = tempdir().expect("temp dir");
        let storage = Storage::Path(dir.path().to_path_buf());

        assert_eq!(storage.read("absent").expect("read"), None);
    }

    #[test]
    fn an_unreadable_value_is_an_error_not_absence() {
        let dir = tempdir().expect("temp dir");
        let storage = Storage::Path(dir.path().to_path_buf());
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
        let storage = Storage::Path(dir.path().to_path_buf());

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
        let storage = Storage::Path(dir.path().to_path_buf());

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
        let storage = Storage::Path(dir.path().to_path_buf());

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
        let storage = Storage::Path(dir.path().to_path_buf());

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
        let storage = Storage::Path(dir.path().to_path_buf());

        storage.write("written", "value").expect("write");
        storage.create("created", "value").expect("create");

        for file in ["written", "created"] {
            let meta = std::fs::metadata(dir.path().join(file)).expect("metadata");
            assert_eq!(meta.permissions().mode() & 0o777, 0o600, "{file}");
        }
    }

    #[test]
    fn hex_round_trips() {
        for bytes in [
            &[][..],
            &[0x00],
            &[0xAB, 0xCD, 0x12, 0x34, 0xFF][..],
            &[b'a', b'Z', 0x09][..],
        ] {
            let text = encode_hex(bytes);
            assert_eq!(decode_hex(&text).expect("decode").as_slice(), bytes);
        }
    }

    #[test]
    fn malformed_hex_is_an_error_not_a_panic() {
        for text in ["a", "zz", "0x00", "€€€", "ab cd"] {
            assert!(decode_hex(text).is_err(), "{text:?} must not decode");
        }
        assert_eq!(decode_hex("").expect("empty hex"), Vec::<u8>::new());
    }
}
