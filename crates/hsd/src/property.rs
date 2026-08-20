use thiserror::Error;

use crate::id::{
    PRIM_ID_BYTES,
    PrimId,
};

const TAG_ATTRIBUTE: u8 = 0;
const TAG_RELATIONSHIP: u8 = 1;

const PARENT_ROOT: u8 = 0;
const PARENT_PRIM: u8 = 1;

#[derive(Error, Debug)]
pub enum PropertyError {
    #[error("empty payload")]
    Empty,
    #[error("unknown tag {0}")]
    Tag(u8),
    #[error("expected {PRIM_ID_BYTES} id bytes, got {0}")]
    IdLength(usize),
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

/// Either typed data or a reference to another prim, as in USD.
///
/// Both share one namespace on the prim, told apart by a leading tag byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Property {
    Attribute(Vec<u8>),
    Relationship(PrimId),
}

impl Property {
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

/// A prim's place in the tree. Never encodes empty: an empty entry reads as
/// absence on every peer, which is how deletion is spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parent {
    Root,
    Prim(PrimId),
}

impl Parent {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Root => vec![PARENT_ROOT],
            Self::Prim(id) => {
                let mut out = Vec::with_capacity(PRIM_ID_BYTES + 1);
                out.push(PARENT_PRIM);
                out.extend_from_slice(&id.0);
                out
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PropertyError> {
        let (tag, rest) = bytes.split_first().ok_or(PropertyError::Empty)?;
        match *tag {
            PARENT_ROOT => Ok(Self::Root),
            PARENT_PRIM => {
                let bytes: [u8; PRIM_ID_BYTES] = rest
                    .try_into()
                    .map_err(|_| PropertyError::IdLength(rest.len()))?;
                Ok(Self::Prim(PrimId(bytes)))
            }
            other => Err(PropertyError::Tag(other)),
        }
    }

    #[must_use]
    pub const fn prim(&self) -> Option<PrimId> {
        match self {
            Self::Root => None,
            Self::Prim(id) => Some(*id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_round_trips() {
        let prop = Property::Attribute(vec![1, 2, 3]);
        assert_eq!(Property::decode(&prop.encode()).expect("decode"), prop);
    }

    #[test]
    fn unknown_attribute_payload_survives() {
        let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let prop = Property::Attribute(payload.clone());
        let decoded = Property::decode(&prop.encode()).expect("decode");
        assert_eq!(decoded.as_attribute(), Some(&payload));
    }

    #[test]
    fn relationship_round_trips() {
        let prop = Property::Relationship(PrimId([9; PRIM_ID_BYTES]));
        assert_eq!(Property::decode(&prop.encode()).expect("decode"), prop);
    }

    #[test]
    fn empty_attribute_is_still_tagged() {
        let encoded = Property::Attribute(Vec::new()).encode();
        assert_eq!(encoded.len(), 1);
        assert_eq!(
            Property::decode(&encoded).expect("decode"),
            Property::Attribute(Vec::new())
        );
    }

    #[test]
    fn parent_never_encodes_empty() {
        assert_ne!(Parent::Root.encode().len(), 0);
        assert_ne!(Parent::Prim(PrimId([2; PRIM_ID_BYTES])).encode().len(), 0);
    }

    #[test]
    fn parent_round_trips() {
        for parent in [Parent::Root, Parent::Prim(PrimId([4; PRIM_ID_BYTES]))] {
            assert_eq!(Parent::decode(&parent.encode()).expect("decode"), parent);
        }
    }

    #[test]
    fn decoding_empty_fails() {
        assert!(Property::decode(&[]).is_err());
        assert!(Parent::decode(&[]).is_err());
    }
}
