use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    attributes::Attribute,
    id::DocId,
};

/// A prim that stands for another document.
///
/// Thirty-two bytes rather than the target's whole package: the target syncs
/// as a namespace like any other document, so a hundred chairs are one
/// document and a hundred references. It resolves to the target's current
/// state — a reference is not pinned to a version, which is a possible later
/// option and not a default.
///
/// The referencing document's opinions override the target's, because the
/// reference chain *is* the layer order. That is what makes recolouring a
/// couch someone else authored expressible at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReferenceAttr(pub DocId);

impl Attribute for ReferenceAttr {
    const KEY: &'static str = "ref";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reference_is_thirty_two_bytes_and_nothing_else() {
        let value = ReferenceAttr(DocId([7; 32]));
        let bytes = value.encode().expect("encode");

        assert_eq!(
            bytes.len(),
            32,
            "the whole point is that a reference costs an id, not a package"
        );
        assert_eq!(ReferenceAttr::decode(&bytes).expect("decode"), value);
    }
}
