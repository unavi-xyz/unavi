use iroh_blobs::Hash;
use iroh_docs::NamespaceId;
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use smol_str::SmolStr;
use xdid::core::did::Did;

use crate::signed_bytes::Signable;

pub mod keys;

/// Format version prefixed to every entry blob. Bump on incompatible change;
/// readers ignore versions they do not understand.
const FORMAT_VERSION: u32 = 0;

/// Encodes a typed value as an entry blob: a leading varint format version
/// followed by the postcard body.
pub fn encode<T: Serialize>(value: &T) -> postcard::Result<Vec<u8>> {
    let mut out = postcard::to_stdvec(&FORMAT_VERSION)?;
    out.extend_from_slice(&postcard::to_stdvec(value)?);
    Ok(out)
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("unsupported format version {0}")]
    Version(u32),
    #[error(transparent)]
    Postcard(#[from] postcard::Error),
}

/// Decodes an entry blob produced by [`encode`], rejecting unknown versions.
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, DecodeError> {
    let (version, rest) = postcard::take_from_bytes::<u32>(bytes)?;
    if version != FORMAT_VERSION {
        return Err(DecodeError::Version(version));
    }
    Ok(postcard::from_bytes(rest)?)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    pub name:   Option<SmolStr>,
    pub bio:    Option<SmolStr>,
    pub avatar: Option<Hash>,
}

/// Binds an iroh-docs author key to a DID, signed by the DID's key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorBinding {
    pub author_pk: [u8; 32],
    pub did:       Did,
    pub sig:       Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avatar {
    pub name: Option<SmolStr>,
    pub vrm:  Hash,
}

/// Reference to a shared space doc from a root doc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceRef {
    pub ns:   NamespaceId,
    pub name: Option<SmolStr>,
}

/// Registry entry announcing a populated space.
///
/// Signed by the announcing DID and relayed verbatim by a server into its
/// registry doc, so readers verify authenticity end-to-end rather than trusting
/// the server's aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beacon {
    pub did:      Did,
    pub endpoint: [u8; 32],
    pub space:    NamespaceId,
    pub expires:  i64,
}

impl Signable for Beacon {}
