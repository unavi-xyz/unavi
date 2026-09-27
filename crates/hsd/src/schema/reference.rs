use const_format::concatcp;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    id::{
        DocId,
        PrimId,
    },
    key,
    prop_name,
    property::{
        Property,
        name::PropName,
    },
};

pub const GROUP: &str = "ref";

const LAYER: &str = "layer";

/// A prim that references another document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReferenceAttr(pub DocId);

impl Property for ReferenceAttr {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/target"));
}

/// The referencing document's opinion on property `name` of prim `target`
/// in the referenced document, stored under the referencing prim `site`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerKey {
    pub target: PrimId,
    pub name:   PropName,
}

impl LayerKey {
    #[must_use]
    pub fn key(&self, site: PrimId) -> String {
        key::Key::nested(
            site,
            GROUP,
            &format!("{LAYER}/{}/{}", self.target, self.name),
        )
        .to_string()
    }

    /// Reads the tail of a [`key::Key::Nested`] under the `ref` group.
    #[must_use]
    pub fn parse(tail: &str) -> Option<Self> {
        let rest = tail.strip_prefix(LAYER)?.strip_prefix('/')?;
        let (target, name) = rest.split_once('/')?;
        Some(Self {
            target: target.parse().ok()?,
            name:   name.parse().ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        property::Payload,
        schema::xform::XformAttr,
    };

    fn site() -> PrimId {
        PrimId([1; 16])
    }

    fn layer_key() -> LayerKey {
        LayerKey {
            target: PrimId([2; 16]),
            name:   XformAttr::NAME,
        }
    }

    #[test]
    fn round_trips_a_layer_key() {
        let raw = layer_key().key(site());
        match key::Key::parse(&raw) {
            Some(key::Key::Nested { prim, group, tail }) => {
                assert_eq!(prim, site());
                assert_eq!(group, GROUP);
                assert_eq!(LayerKey::parse(&tail), Some(layer_key()));
            }
            other => panic!("expected a nested key, got {other:?}"),
        }
    }

    #[test]
    fn deleting_the_namespace_sweeps_target_and_layer() {
        let prefix = format!("{}{}/", key::prim_prefix(site()), GROUP);
        assert!(
            key::Key::prop(site(), &ReferenceAttr::NAME)
                .to_string()
                .starts_with(&prefix)
        );
        assert!(layer_key().key(site()).starts_with(&prefix));
    }

    #[test]
    fn a_layer_spine_is_not_a_layer_key() {
        assert_eq!(LayerKey::parse(LAYER), None);
        assert_eq!(
            LayerKey::parse(&format!("{LAYER}/{}", PrimId([2; 16]))),
            None
        );
    }

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
