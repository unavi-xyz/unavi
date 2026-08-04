//! `.hsdz`: a compiled document as one self-contained blob.
//!
//! Bulk is inlined rather than referenced by hash. A package holding only
//! hashes would be unresolvable across the network, since those blobs are not
//! entries of the document that carries the package and so are never pinned by
//! a replicating host nor fetched by a syncing peer.

use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::{
    meta::VERSION,
    state::entry::EntryValue,
};

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

/// Splits a package's entries into inline values and bulk payloads.
///
/// Unpacking is `add_bytes` each bulk payload into the blob store, then apply
/// every entry — the same shape as loading a live document, with the fetch
/// already done.
#[must_use]
pub fn split(package: Package) -> (Vec<(String, Vec<u8>)>, Vec<(String, Vec<u8>)>) {
    package
        .entries
        .into_iter()
        .partition(|(key, _)| !key.starts_with(crate::key::BULK_PREFIX))
}

/// The value an inline entry contributes when applied.
#[must_use]
pub const fn inline_value(bytes: Vec<u8>) -> EntryValue {
    EntryValue::Bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> Package {
        let mut entries = BTreeMap::new();
        entries.insert("p/A/xform/".to_owned(), vec![0, 1, 2]);
        entries.insert("b/A/mesh:POSITION/".to_owned(), vec![9; 64]);
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

    #[test]
    fn split_separates_bulk() {
        let (inline, bulk) = split(package());
        assert_eq!(inline.len(), 2);
        assert_eq!(bulk.len(), 1);
        assert!(bulk[0].0.starts_with("b/"));
    }
}
