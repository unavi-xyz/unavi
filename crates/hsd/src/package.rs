//! `.hsdz`: a compiled document and everything it references, as one
//! self-contained blob.
//!
//! A package is entries — no bloom store to reconcile during replication, no
//! published set to preserve. Documents the root references travel beside it
//! rather than nested inside its prims, so a reference costs 32 bytes in the
//! root and the target is stored once however many prims name it.

use std::collections::{
    BTreeMap,
    HashMap,
};

use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::{
    attributes::{
        Attribute,
        reference::ReferenceAttr,
    },
    id::DocId,
    key,
    meta::VERSION,
    property::{
        Property,
        PropertyError,
    },
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
    #[error("property {0}")]
    Property(#[from] PropertyError),
    #[error("reference to {0}, which the package does not carry")]
    Dangling(DocId),
}

/// Entries sorted by key, so an unchanged input compiles to identical bytes
/// and its hash is stable across rebuilds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub version:   u16,
    /// The root document.
    pub entries:   Vec<(String, Vec<u8>)>,
    /// Every document referenced from the root or from another of these, under
    /// the placeholder id its `ref` entries name. Flat rather than nested, so
    /// minting is one pass and two prims naming one file share a document.
    pub documents: Vec<(DocId, Vec<(String, Vec<u8>)>)>,
}

impl Package {
    #[must_use]
    pub fn new(entries: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            version:   VERSION,
            entries:   entries.into_iter().collect(),
            documents: Vec::new(),
        }
    }

    /// The placeholder a compiled file's document is carried under.
    ///
    /// Derived from the file's identity rather than minted, so compiling twice
    /// gives the same package bytes and two prims naming one file resolve to
    /// one entry.
    #[must_use]
    pub fn placeholder(source: &str) -> DocId {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"hsd:package-doc");
        hasher.update(source.as_bytes());
        DocId(*hasher.finalize().as_bytes())
    }

    /// Rewrites every `ref` value in `entries` through `minted`.
    ///
    /// A placeholder is meaningless outside the package that carries it, so a
    /// reference the map does not answer is an error rather than a value
    /// written through: it would name a namespace nobody can ever serve.
    pub fn rewrite_refs(
        entries: &mut [(String, Vec<u8>)],
        minted: &HashMap<DocId, DocId>,
    ) -> Result<(), PackageError> {
        for (raw, value) in entries {
            let Some(key::Key::Prop { name, .. }) = key::parse(raw) else {
                continue;
            };
            if name != ReferenceAttr::KEY {
                continue;
            }
            let Property::Attribute(payload) = Property::decode(value)? else {
                continue;
            };
            let placeholder = ReferenceAttr::decode(&payload)?.0;
            let target = minted
                .get(&placeholder)
                .copied()
                .ok_or(PackageError::Dangling(placeholder))?;
            *value = Property::Attribute(ReferenceAttr(target).encode()?).encode();
        }
        Ok(())
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
