//! Key layout.
//!
//! iroh-docs applies Willow prefix semantics: an entry removes all of its
//! author's older entries under its own key as a prefix. Two rules follow, and
//! both are enforced here rather than by convention.
//!
//! 1. No data lives at a key that prefixes another key, so the parent is a
//!    reserved property (`p/<prim>/parent/`) and never `p/<prim>/` itself, and
//!    the reference target and reference layer sit below reserved segments
//!    rather than at `p/<prim>/ref/`.
//! 2. Every key ends with `/`, so `mesh:index/` cannot prefix `mesh:indices/`.
//!
//! A prim's reference data is self-contained under its own prefix:
//!
//! ```text
//! p/<site>/ref/target/                 the ref property value (the target id)
//! p/<site>/ref/layer/<target>/<name>/  opinions about the referenced document
//! ```
//!
//! `p/<site>/ref/` and `p/<site>/ref/layer/` are pure spines. Deleting the
//! prim therefore sweeps its target and its whole reference layer with it.

use smol_str::SmolStr;

use crate::id::PrimId;

pub const META: &str = "meta/";
pub const PRIM_PREFIX: &str = "p/";
pub const PARENT: &str = "parent";
/// The property name carrying a prim's reference target.
pub const REF: &str = "ref";
/// Segment under `p/<site>/ref/` holding the target value.
pub const TARGET: &str = "target";
/// Segment under `p/<site>/ref/` holding the reference layer.
pub const LAYER: &str = "layer";

/// Every top-level prefix a reader has to ask for to hold a whole document.
pub const PREFIXES: [&str; 2] = [META, PRIM_PREFIX];

#[must_use]
pub fn prim_prefix(prim: PrimId) -> String {
    format!("{PRIM_PREFIX}{prim}/")
}

/// The key for a prim's property.
///
/// The reference target is an ordinary property named `ref`, but its wire key
/// is structural — [`ref_target`] — so the reference layer can live beneath a
/// sibling segment instead of under the target value.
#[must_use]
pub fn prop(prim: PrimId, name: &str) -> String {
    if name == REF {
        return ref_target(prim);
    }
    format!("{PRIM_PREFIX}{prim}/{name}/")
}

#[must_use]
pub fn parent(prim: PrimId) -> String {
    prop(prim, PARENT)
}

/// The reference target: `p/<prim>/ref/target/`. Data, so it prefixes nothing.
#[must_use]
pub fn ref_target(prim: PrimId) -> String {
    format!("{PRIM_PREFIX}{prim}/{REF}/{TARGET}/")
}

/// `p/<site>/ref/layer/`, the spine above a site's reference-layer keys.
#[must_use]
pub fn ref_layer_prefix(site: PrimId) -> String {
    format!("{PRIM_PREFIX}{site}/{REF}/{LAYER}/")
}

/// Keyed by site rather than by target: two prims referencing one couch have
/// to be recolourable separately.
#[must_use]
pub fn ref_layer_key(site: PrimId, target: PrimId, name: &str) -> String {
    format!("{PRIM_PREFIX}{site}/{REF}/{LAYER}/{target}/{name}/")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Meta,
    Prop {
        prim: PrimId,
        name: SmolStr,
    },
    /// A prim's reference target, spelled `p/<site>/ref/target/` rather than
    /// as a property so the reference layer can sit beside it.
    RefTarget {
        site: PrimId,
    },
    /// An opinion about a referenced document's prim. `p/` keeps meaning "my
    /// own prims", so a document never appears to contain prims it does not
    /// have.
    RefLayer {
        site:   PrimId,
        target: PrimId,
        name:   SmolStr,
    },
}

/// Parses a document key, returning `None` for anything this format does not
/// define. Unrecognized keys are ignored rather than rejected, so a client can
/// sync a document written by a newer one.
#[must_use]
pub fn parse(key: &str) -> Option<Key> {
    if key == META {
        return Some(Key::Meta);
    }

    let rest = key.strip_prefix(PRIM_PREFIX)?;
    let rest = rest.strip_suffix('/')?;
    let (prim, tail) = rest.split_once('/')?;
    let prim = prim.parse::<PrimId>().ok()?;

    // `ref/` is a pure spine: the target is at `ref/target/` and the layer at
    // `ref/layer/...`, so nothing parses at `ref` itself.
    if tail == REF {
        return None;
    }
    if let Some(rest) = tail.strip_prefix(REF).and_then(|r| r.strip_prefix('/')) {
        if rest == TARGET {
            return Some(Key::RefTarget { site: prim });
        }
        if let Some(rest) = rest.strip_prefix(LAYER).and_then(|r| r.strip_prefix('/')) {
            let (target, name) = rest.split_once('/')?;
            if is_valid_name(name) {
                return Some(Key::RefLayer {
                    site:   prim,
                    target: target.parse().ok()?,
                    name:   SmolStr::new(name),
                });
            }
        }
        return None;
    }

    if !is_valid_name(tail) {
        return None;
    }

    Some(Key::Prop {
        prim,
        name: SmolStr::new(tail),
    })
}

/// A property name may not be empty or contain a `/`, since either would let
/// one key prefix another.
#[must_use]
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> PrimId {
        PrimId([1; 16])
    }

    #[test]
    fn round_trips_a_property() {
        let key = prop(id(), "material:binding");
        assert_eq!(
            parse(&key),
            Some(Key::Prop {
                prim: id(),
                name: SmolStr::new("material:binding"),
            })
        );
    }

    #[test]
    fn round_trips_a_reference_target() {
        assert_eq!(
            parse(&ref_target(id())),
            Some(Key::RefTarget { site: id() })
        );
    }

    #[test]
    fn the_ref_property_key_is_the_reference_target() {
        assert_eq!(prop(id(), REF), ref_target(id()));
    }

    #[test]
    fn round_trips_a_reference_layer_key() {
        let target = PrimId([2; 16]);
        let key = ref_layer_key(id(), target, "material:base_color");
        assert_eq!(
            parse(&key),
            Some(Key::RefLayer {
                site: id(),
                target,
                name: SmolStr::new("material:base_color"),
            })
        );
    }

    #[test]
    fn a_reference_layer_key_is_not_read_as_a_prim_property() {
        let key = ref_layer_key(id(), PrimId([2; 16]), "xform");
        assert!(key.starts_with(PRIM_PREFIX));
        assert!(matches!(parse(&key), Some(Key::RefLayer { .. })));
    }

    #[test]
    fn ref_spines_are_not_keys() {
        assert_eq!(parse(&format!("{PRIM_PREFIX}{}/ref/", id())), None);
        assert_eq!(parse(&ref_layer_prefix(id())), None);
        assert_eq!(parse(&format!("{PRIM_PREFIX}{}/ref/target", id())), None);
        assert_eq!(
            parse(&format!(
                "{PRIM_PREFIX}{}/ref/layer/{}/",
                id(),
                PrimId([2; 16])
            )),
            None
        );
    }

    #[test]
    fn a_name_sharing_the_ref_prefix_is_still_a_property() {
        assert_eq!(
            parse(&format!("{PRIM_PREFIX}{}/reference/", id())),
            Some(Key::Prop {
                prim: id(),
                name: SmolStr::new("reference"),
            })
        );
    }

    #[test]
    fn round_trips_meta() {
        assert_eq!(parse(META), Some(Key::Meta));
    }

    #[test]
    fn bare_prim_prefix_is_not_a_key() {
        assert_eq!(parse(&prim_prefix(id())), None);
    }

    #[test]
    fn missing_trailing_slash_is_rejected() {
        assert_eq!(parse(&format!("p/{}/xform", id())), None);
    }

    #[test]
    fn a_nested_property_name_is_rejected() {
        assert_eq!(parse(&format!("p/{}/mesh/POSITION/", id())), None);
    }

    #[test]
    fn parent_is_a_property_not_the_prim_key() {
        let parent_key = parent(id());
        assert!(parent_key.starts_with(&prim_prefix(id())));
        assert_ne!(parent_key, prim_prefix(id()));
    }
}
