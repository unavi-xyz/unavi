use serde::{
    Deserialize,
    Serialize,
};
use smol_str::SmolStr;

use crate::{
    attributes::Attribute,
    id::{
        DocId,
        PrimId,
    },
    key,
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
///
/// This is an ordinary property named `ref`, composed through the stack like
/// any other, but its wire key is structural: the value is written at
/// `p/<site>/ref/target/` and the referencing document's opinions at
/// `p/<site>/ref/layer/<target>/<name>/`, so deleting the prim sweeps both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReferenceAttr(pub DocId);

impl Attribute for ReferenceAttr {
    const KEY: &'static str = "ref";
}

/// Segment holding the target value.
const TARGET: &str = "target";
/// Segment above the reference layer.
const LAYER: &str = "layer";

/// What a key inside `ref`'s namespace names. Nothing parses at the namespace
/// key itself: it is a spine, so that holding data there would prefix-delete
/// everything below it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefKey {
    Target,
    Layer { target: PrimId, name: SmolStr },
}

/// The reference target: `p/<site>/ref/target/`. Data, so it prefixes nothing.
#[must_use]
pub fn target_key(site: PrimId) -> String {
    key::prop_sub(site, ReferenceAttr::KEY, TARGET)
}

/// `p/<site>/ref/layer/`, the spine above a site's reference-layer keys.
#[must_use]
pub fn layer_prefix(site: PrimId) -> String {
    format!("{}{LAYER}/", key::prop(site, ReferenceAttr::KEY))
}

/// Keyed by site rather than by target: two prims referencing one couch have
/// to be recolourable separately.
#[must_use]
pub fn layer_key(site: PrimId, target: PrimId, name: &str) -> String {
    key::prop_sub(
        site,
        ReferenceAttr::KEY,
        &format!("{LAYER}/{target}/{name}"),
    )
}

/// Reads the tail [`key::Key::PropSub`] hands back for a `ref` key.
#[must_use]
pub fn parse_tail(tail: &str) -> Option<RefKey> {
    if tail == TARGET {
        return Some(RefKey::Target);
    }
    let rest = tail.strip_prefix(LAYER)?.strip_prefix('/')?;
    let (target, name) = rest.split_once('/')?;
    if !key::is_valid_name(name) {
        return None;
    }
    Some(RefKey::Layer {
        target: target.parse().ok()?,
        name:   SmolStr::new(name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site() -> PrimId {
        PrimId([1; 16])
    }

    fn target() -> PrimId {
        PrimId([2; 16])
    }

    fn tail_of(key: &str) -> SmolStr {
        match key::parse(key) {
            Some(key::Key::PropSub { name, tail, .. }) if name == ReferenceAttr::KEY => tail,
            other => panic!("expected a ref sub-key, got {other:?}"),
        }
    }

    #[test]
    fn round_trips_a_target_key() {
        assert_eq!(
            parse_tail(&tail_of(&target_key(site()))),
            Some(RefKey::Target)
        );
    }

    #[test]
    fn round_trips_a_layer_key() {
        let key = layer_key(site(), target(), "material:base_color");
        assert_eq!(
            parse_tail(&tail_of(&key)),
            Some(RefKey::Layer {
                target: target(),
                name:   SmolStr::new("material:base_color"),
            })
        );
    }

    /// The layer holds opinions about another document's prims, so its keys
    /// must not read as prims of this one.
    #[test]
    fn a_layer_key_is_not_read_as_a_property_of_the_site() {
        let key = layer_key(site(), target(), "xform");
        assert!(matches!(
            key::parse(&key),
            Some(key::Key::PropSub { prim, .. }) if prim == site()
        ));
    }

    #[test]
    fn deleting_the_prim_sweeps_both() {
        let prefix = key::prim_prefix(site());
        assert!(target_key(site()).starts_with(&prefix));
        assert!(layer_key(site(), target(), "xform").starts_with(&prefix));
    }

    #[test]
    fn spines_are_not_keys() {
        assert_eq!(parse_tail(""), None);
        assert_eq!(parse_tail(LAYER), None);
        assert_eq!(parse_tail(&format!("{LAYER}/{}", target())), None);
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
