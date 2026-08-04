//! Key layout.
//!
//! iroh-docs applies Willow prefix semantics: an entry removes all of its
//! author's older entries under its own key as a prefix. Two rules follow, and
//! both are enforced here rather than by convention.
//!
//! 1. No data lives at a key that prefixes another key, so the parent is a
//!    reserved property (`p/<prim>/parent/`) and never `p/<prim>/` itself.
//! 2. Every key ends with `/`, so `mesh:index/` cannot prefix `mesh:indices/`.

use smol_str::SmolStr;

use crate::id::PrimId;

pub const META: &str = "meta/";
pub const PRIM_PREFIX: &str = "p/";
pub const BULK_PREFIX: &str = "b/";
pub const PARENT: &str = "parent";

#[must_use]
pub fn prim_prefix(prim: PrimId) -> String {
    format!("{PRIM_PREFIX}{prim}/")
}

#[must_use]
pub fn bulk_prefix(prim: PrimId) -> String {
    format!("{BULK_PREFIX}{prim}/")
}

#[must_use]
pub fn prop(prim: PrimId, name: &str) -> String {
    format!("{PRIM_PREFIX}{prim}/{name}/")
}

#[must_use]
pub fn bulk(prim: PrimId, slot: &str) -> String {
    format!("{BULK_PREFIX}{prim}/{slot}/")
}

#[must_use]
pub fn parent(prim: PrimId) -> String {
    prop(prim, PARENT)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Meta,
    Prop { prim: PrimId, name: SmolStr },
    Bulk { prim: PrimId, slot: SmolStr },
}

/// Parses a document key, returning `None` for anything this format does not
/// define. Unrecognized keys are ignored rather than rejected, so a client can
/// sync a document written by a newer one.
#[must_use]
pub fn parse(key: &str) -> Option<Key> {
    if key == META {
        return Some(Key::Meta);
    }

    let (prefix, rest) = if let Some(rest) = key.strip_prefix(PRIM_PREFIX) {
        (PRIM_PREFIX, rest)
    } else {
        (BULK_PREFIX, key.strip_prefix(BULK_PREFIX)?)
    };

    let rest = rest.strip_suffix('/')?;
    let (prim, name) = rest.split_once('/')?;
    let prim = prim.parse::<PrimId>().ok()?;
    if !is_valid_name(name) {
        return None;
    }

    Some(if prefix == PRIM_PREFIX {
        Key::Prop {
            prim,
            name: SmolStr::new(name),
        }
    } else {
        Key::Bulk {
            prim,
            slot: SmolStr::new(name),
        }
    })
}

/// A property or slot name may not be empty or contain a `/`, since either
/// would let one key prefix another.
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
    fn round_trips_bulk() {
        let key = bulk(id(), "mesh:POSITION");
        assert_eq!(
            parse(&key),
            Some(Key::Bulk {
                prim: id(),
                slot: SmolStr::new("mesh:POSITION"),
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
        assert_eq!(parse(&bulk_prefix(id())), None);
    }

    #[test]
    fn missing_trailing_slash_is_rejected() {
        assert_eq!(parse(&format!("p/{}/xform", id())), None);
    }

    #[test]
    fn nested_slot_spelling_is_rejected() {
        assert_eq!(parse(&format!("p/{}/mesh/POSITION/", id())), None);
    }

    #[test]
    fn slot_names_do_not_prefix_siblings() {
        let index = bulk(id(), "mesh:index");
        let indices = bulk(id(), "mesh:indices");
        assert!(!indices.starts_with(&index));
    }

    #[test]
    fn parent_is_a_property_not_the_prim_key() {
        let parent_key = parent(id());
        assert!(parent_key.starts_with(&prim_prefix(id())));
        assert_ne!(parent_key, prim_prefix(id()));
    }
}
