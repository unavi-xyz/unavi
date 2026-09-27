use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    id::PrimId,
    prop_name,
    property::{
        Payload,
        Property,
        name::PropName,
        value::{
            PropertyError,
            Value,
        },
    },
};

pub const GROUP: &str = "parent";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParentAttr {
    Root,
    Prim(PrimId),
}

impl Property for ParentAttr {
    const NAME: PropName = prop_name!(GROUP);
}

impl ParentAttr {
    #[must_use]
    pub const fn prim(&self) -> Option<PrimId> {
        match self {
            Self::Root => None,
            Self::Prim(id) => Some(*id),
        }
    }

    #[must_use]
    pub fn to_wire(parent: Option<Self>) -> Vec<u8> {
        parent.map_or_else(Vec::new, |parent| {
            Value::Attribute(parent.encode().expect("a parent always encodes")).encode()
        })
    }

    pub fn from_wire(bytes: &[u8]) -> Result<Option<Self>, PropertyError> {
        if bytes.is_empty() {
            return Ok(None);
        }
        let payload = Value::decode(bytes)?;
        let payload = payload.as_attribute().ok_or(PropertyError::NotAttribute)?;
        Ok(Some(Self::decode(payload)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_the_wire() {
        for parent in [
            None,
            Some(ParentAttr::Root),
            Some(ParentAttr::Prim(PrimId([4; 16]))),
        ] {
            assert_eq!(
                ParentAttr::from_wire(&ParentAttr::to_wire(parent)).expect("decode"),
                parent
            );
        }
    }

    #[test]
    fn a_stated_parent_never_encodes_empty() {
        assert_ne!(ParentAttr::to_wire(Some(ParentAttr::Root)), Vec::new());
        assert_ne!(
            ParentAttr::to_wire(Some(ParentAttr::Prim(PrimId([2; 16])))),
            Vec::new()
        );
    }

    #[test]
    fn a_relationship_is_not_a_parent() {
        let bytes = Value::Relationship(PrimId([1; 16])).encode();
        assert!(ParentAttr::from_wire(&bytes).is_err());
    }
}
