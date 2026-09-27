use std::fmt::Debug;

use thiserror::Error;

use crate::id::{
    PRIM_ID_BYTES,
    PrimId,
};

const TAG_ATTRIBUTE: u8 = 0;
const TAG_RELATIONSHIP: u8 = 1;

#[derive(Error, Debug)]
pub enum PropertyError {
    #[error("empty payload")]
    Empty,
    #[error("unknown tag {0}")]
    Tag(u8),
    #[error("expected {PRIM_ID_BYTES} id bytes, got {0}")]
    IdLength(usize),
    #[error("expected an attribute, got a relationship")]
    NotAttribute,
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

/// Either typed data or a reference to another prim. A leading tag byte
/// tells them apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Attribute(Vec<u8>),
    Relationship(PrimId),
}

impl Value {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Attribute(payload) => {
                let mut out = Vec::with_capacity(payload.len() + 1);
                out.push(TAG_ATTRIBUTE);
                out.extend_from_slice(payload);
                out
            }
            Self::Relationship(target) => {
                let mut out = Vec::with_capacity(PRIM_ID_BYTES + 1);
                out.push(TAG_RELATIONSHIP);
                out.extend_from_slice(&target.0);
                out
            }
        }
    }

    /// Hashes the bytes [`Self::encode`] would produce without building them.
    pub fn hash_into(&self, hasher: &mut blake3::Hasher) {
        match self {
            Self::Attribute(payload) => {
                hasher.update(&[TAG_ATTRIBUTE]);
                hasher.update(payload);
            }
            Self::Relationship(target) => {
                hasher.update(&[TAG_RELATIONSHIP]);
                hasher.update(&target.0);
            }
        }
    }

    /// Hashes an attribute payload without wrapping it in a `Vec`.
    pub fn hash_attribute_into(hasher: &mut blake3::Hasher, payload: &[u8]) {
        hasher.update(&[TAG_ATTRIBUTE]);
        hasher.update(payload);
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PropertyError> {
        let (tag, rest) = bytes.split_first().ok_or(PropertyError::Empty)?;
        match *tag {
            TAG_ATTRIBUTE => Ok(Self::Attribute(rest.to_vec())),
            TAG_RELATIONSHIP => {
                let bytes: [u8; PRIM_ID_BYTES] = rest
                    .try_into()
                    .map_err(|_| PropertyError::IdLength(rest.len()))?;
                Ok(Self::Relationship(PrimId(bytes)))
            }
            other => Err(PropertyError::Tag(other)),
        }
    }

    #[must_use]
    pub const fn as_attribute(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Attribute(payload) => Some(payload),
            Self::Relationship(_) => None,
        }
    }

    #[must_use]
    pub const fn as_relationship(&self) -> Option<PrimId> {
        match self {
            Self::Relationship(target) => Some(*target),
            Self::Attribute(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_round_trips() {
        let prop = Value::Attribute(vec![1, 2, 3]);
        assert_eq!(Value::decode(&prop.encode()).expect("decode"), prop);
    }

    #[test]
    fn relationship_round_trips() {
        let prop = Value::Relationship(PrimId([9; PRIM_ID_BYTES]));
        assert_eq!(Value::decode(&prop.encode()).expect("decode"), prop);
    }

    #[test]
    fn empty_attribute_is_still_tagged() {
        let encoded = Value::Attribute(Vec::new()).encode();
        assert_eq!(encoded.len(), 1);
        assert_eq!(
            Value::decode(&encoded).expect("decode"),
            Value::Attribute(Vec::new())
        );
    }

    #[test]
    fn decoding_empty_fails() {
        assert!(Value::decode(&[]).is_err());
    }
}
