//! `.hsdz` self-contained packaged file format.

use std::collections::{
    BTreeMap,
    HashMap,
};

use bytes::Bytes;
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::{
    bounds::{
        MAX_ENTRY_BYTES,
        MAX_PACKAGE_BYTES,
        MAX_PACKAGE_DOCUMENTS,
    },
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
    #[error("package is {0} bytes, over the cap of {MAX_PACKAGE_BYTES}")]
    TooLarge(usize),
    #[error("package carries {0} documents, over the cap of {MAX_PACKAGE_DOCUMENTS}")]
    TooManyDocuments(usize),
    #[error("entry {key} is {len} bytes, over the cap of {MAX_ENTRY_BYTES}")]
    EntryTooLarge { key: String, len: usize },
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
            let Value::Attribute(payload) = Value::decode(&Bytes::copy_from_slice(value))? else {
                continue;
            };
            let placeholder = ReferenceAttr::decode(&payload)?.0;
            let target = minted
                .get(&placeholder)
                .copied()
                .ok_or(PackageError::Dangling(placeholder))?;
            *value = Value::Attribute(ReferenceAttr(target).encode()?.into()).encode();
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, PackageError> {
        let mut out = MAGIC.to_vec();
        out.extend(postcard::to_stdvec(self)?);
        Ok(out)
    }

    /// Refuses a package past the byte, entry or document caps in
    /// [`crate::bounds`].
    pub fn decode(bytes: &[u8]) -> Result<Self, PackageError> {
        if bytes.len() > MAX_PACKAGE_BYTES {
            return Err(PackageError::TooLarge(bytes.len()));
        }
        let body = bytes.strip_prefix(MAGIC).ok_or(PackageError::Magic)?;
        let package = postcard::from_bytes::<Self>(body)?;
        if package.format_version > DOC_VERSION {
            return Err(PackageError::Version(package.format_version));
        }
        if package.documents.len() > MAX_PACKAGE_DOCUMENTS {
            return Err(PackageError::TooManyDocuments(package.documents.len()));
        }

        let every_entry = package
            .entries
            .iter()
            .chain(package.documents.iter().flat_map(|(_, entries)| entries));
        for (key, value) in every_entry {
            if value.len() > MAX_ENTRY_BYTES {
                return Err(PackageError::EntryTooLarge {
                    key: key.clone(),
                    len: value.len(),
                });
            }
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
    fn foreign_bytes_are_rejected() {
        assert!(Package::decode(b"nope").is_err());
    }

    #[test]
    fn an_oversized_entry_in_a_sub_document_is_rejected() {
        let mut package = package();
        package.documents.push((
            Package::placeholder("child"),
            vec![("p/B/script/".to_owned(), vec![0; MAX_ENTRY_BYTES + 1])],
        ));
        let bytes = package.encode().expect("encode");
        assert!(matches!(
            Package::decode(&bytes),
            Err(PackageError::EntryTooLarge { .. })
        ));
    }

    #[test]
    fn a_package_past_the_document_cap_is_rejected() {
        let mut package = package();
        package.documents = (0..=MAX_PACKAGE_DOCUMENTS)
            .map(|i| (Package::placeholder(&i.to_string()), Vec::new()))
            .collect();
        let bytes = package.encode().expect("encode");
        assert!(matches!(
            Package::decode(&bytes),
            Err(PackageError::TooManyDocuments(_))
        ));
    }

    #[test]
    fn newer_versions_are_rejected() {
        let mut package = package();
        package.format_version = DOC_VERSION + 1;
        let bytes = package.encode().expect("encode");
        assert!(Package::decode(&bytes).is_err());
    }
}
