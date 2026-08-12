//! `.hsdz`: a compiled document as one self-contained blob.
//!
//! A package is only the document's entries — no bloom store to reconcile
//! during replication, no published set to preserve.

use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::meta::VERSION;

pub const MAGIC: &[u8; 4] = b"HSDZ";
pub const EXTENSION: &str = "hsdz";

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("not an hsdz package")]
    Magic,
    #[error("unsupported package version {0}")]
    Version(u16),
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

/// Entries sorted by key, so an unchanged input compiles to identical bytes
/// and its hash is stable across rebuilds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub version: u16,
    pub entries: Vec<(String, Vec<u8>)>,
}

impl Package {
    #[must_use]
    pub fn new(entries: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            version: VERSION,
            entries: entries.into_iter().collect(),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, PackageError> {
        let mut out = MAGIC.to_vec();
        out.extend(postcard::to_stdvec(self)?);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PackageError> {
        let body = bytes.strip_prefix(MAGIC).ok_or(PackageError::Magic)?;
        let package = postcard::from_bytes::<Self>(body)?;
        if package.version > VERSION {
            return Err(PackageError::Version(package.version));
        }
        Ok(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> Package {
        let mut entries = BTreeMap::new();
        entries.insert("p/A/xform/".to_owned(), vec![0, 1, 2]);
        entries.insert("p/A/mesh:POSITION/".to_owned(), vec![9; 64]);
        entries.insert("meta/".to_owned(), vec![1, 0]);
        Package::new(entries)
    }

    #[test]
    fn round_trips() {
        let original = package();
        let bytes = original.encode().expect("encode");
        assert_eq!(Package::decode(&bytes).expect("decode"), original);
    }

    #[test]
    fn encoding_is_deterministic() {
        assert_eq!(
            package().encode().expect("encode"),
            package().encode().expect("encode")
        );
    }

    #[test]
    fn entries_are_key_sorted() {
        let keys = package()
            .entries
            .iter()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn foreign_bytes_are_rejected() {
        assert!(Package::decode(b"nope").is_err());
    }

    #[test]
    fn newer_versions_are_rejected() {
        let mut package = package();
        package.version = VERSION + 1;
        let bytes = package.encode().expect("encode");
        assert!(Package::decode(&bytes).is_err());
    }
}
