use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    attributes::Attribute,
    id::PrimId,
    property::{
        Property,
        PropertyError,
    },
};

/// A prim's place in the tree, and the record that it is there at all — there
/// is no other key saying a prim exists.
///
/// An ordinary attribute on the wire: the same key shape, the same
/// `Property::Attribute` tag, the same postcard payload. What is structural
/// about it is only what the state layer does after decoding — settling a
/// parent reindexes the prim among its siblings and re-realizes the subtree,
/// where settling any other attribute recomposes one value. That is why
/// `OpinionKey` names it and `HsdState::set_parent` is its writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParentAttr {
    Root,
    Prim(PrimId),
}

impl Attribute for ParentAttr {
    const KEY: &'static str = "parent";
}

impl ParentAttr {
    #[must_use]
    pub const fn prim(&self) -> Option<PrimId> {
        match self {
            Self::Root => None,
            Self::Prim(id) => Some(*id),
        }
    }

    /// The entry value for a parent opinion. `None` encodes empty, which is
    /// how the format spells absence, and is why a stated parent must never
    /// encode to nothing.
    #[must_use]
    pub fn to_wire(parent: Option<Self>) -> Vec<u8> {
        parent.map_or_else(Vec::new, |parent| {
            Property::Attribute(parent.encode().expect("a parent always encodes")).encode()
        })
    }

    /// Reads an entry value back, `None` for the empty value that states
    /// absence.
    pub fn from_wire(bytes: &[u8]) -> Result<Option<Self>, PropertyError> {
        if bytes.is_empty() {
            return Ok(None);
        }
        let payload = Property::decode(bytes)?;
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

    /// An empty value reads as absence on every peer, so a stated parent that
    /// encoded to nothing would read as a deletion.
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
        let bytes = Property::Relationship(PrimId([1; 16])).encode();
        assert!(ParentAttr::from_wire(&bytes).is_err());
    }
}
