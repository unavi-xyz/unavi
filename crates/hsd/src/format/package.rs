//! `.hsdz` self-contained packaged file format.

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
    format::meta::DOC_VERSION,
    id::DocId,
    key,
    property::{
        Payload,
        Property,
        value::{
            PropertyError,
            Value,
        },
    },
    schema::reference::ReferenceAttr,
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
    Value(#[from] PropertyError),
    #[error("reference to {0}, which the package does not carry")]
    Dangling(DocId),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub format_version: u16,
    /// The root document.
    pub entries:        Vec<(String, Vec<u8>)>,
    /// Every document referenced within this file, recursively.
    pub documents:      Vec<(DocId, Vec<(String, Vec<u8>)>)>,
}

impl Package {
    #[must_use]
    pub fn new(entries: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            format_version: DOC_VERSION,
            entries:        entries.into_iter().collect(),
            documents:      Vec::new(),
        }
    }

    /// Derives a placeholder id from the file path, so repeated builds and
    /// repeated references produce the same id.
    #[must_use]
    pub fn placeholder(source: &str) -> DocId {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"hsd:package-doc");
        hasher.update(source.as_bytes());
        DocId(*hasher.finalize().as_bytes())
    }

    /// A reference the map does not answer is an error.
    pub fn rewrite_refs(
        entries: &mut [(String, Vec<u8>)],
        minted: &HashMap<DocId, DocId>,
    ) -> Result<(), PackageError> {
        for (raw, value) in entries {
            let Some(key::Key::Prop { name, .. }) = key::Key::parse(raw) else {
                continue;
            };
            if name != ReferenceAttr::NAME {
                continue;
            }
            let Value::Attribute(payload) = Value::decode(value)? else {
                continue;
            };
            let placeholder = ReferenceAttr::decode(&payload)?.0;
            let target = minted
                .get(&placeholder)
                .copied()
                .ok_or(PackageError::Dangling(placeholder))?;
            *value = Value::Attribute(ReferenceAttr(target).encode()?).encode();
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
        if package.format_version > DOC_VERSION {
            return Err(PackageError::Version(package.format_version));
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
        package.format_version = DOC_VERSION + 1;
        let bytes = package.encode().expect("encode");
        assert!(Package::decode(&bytes).is_err());
    }
}
