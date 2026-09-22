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
pub const OVERRIDE_PREFIX: &str = "o/";
pub const PARENT: &str = "parent";

/// Every top-level prefix a reader has to ask for to hold a whole document.
pub const PREFIXES: [&str; 3] = [META, PRIM_PREFIX, OVERRIDE_PREFIX];

#[must_use]
pub fn prim_prefix(prim: PrimId) -> String {
    format!("{PRIM_PREFIX}{prim}/")
}

#[must_use]
pub fn prop(prim: PrimId, name: &str) -> String {
    format!("{PRIM_PREFIX}{prim}/{name}/")
}

#[must_use]
pub fn parent(prim: PrimId) -> String {
    prop(prim, PARENT)
}

/// Everything this document says about the prims of the document its `site`
/// prim references.
#[must_use]
pub fn override_prefix(site: PrimId) -> String {
    format!("{OVERRIDE_PREFIX}{site}/")
}

/// Keyed by site rather than by target: two prims referencing one couch have
/// to be recolourable separately.
#[must_use]
pub fn override_key(site: PrimId, target: PrimId, name: &str) -> String {
    format!("{OVERRIDE_PREFIX}{site}/{target}/{name}/")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Meta,
    Prop {
        prim: PrimId,
        name: SmolStr,
    },
    /// An opinion about a referenced document's prim. `p/` keeps meaning "my
    /// own prims", so a document never appears to contain prims it does not
    /// have.
    Override {
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

    if let Some(rest) = key.strip_prefix(OVERRIDE_PREFIX) {
        let rest = rest.strip_suffix('/')?;
        let (site, rest) = rest.split_once('/')?;
        let (target, name) = rest.split_once('/')?;
        if !is_valid_name(name) {
            return None;
        }
        return Some(Key::Override {
            site:   site.parse().ok()?,
            target: target.parse().ok()?,
            name:   SmolStr::new(name),
        });
    }

    let rest = key.strip_prefix(PRIM_PREFIX)?;
    let rest = rest.strip_suffix('/')?;
    let (prim, name) = rest.split_once('/')?;
    let prim = prim.parse::<PrimId>().ok()?;
    if !is_valid_name(name) {
        return None;
    }

    Some(Key::Prop {
        prim,
        name: SmolStr::new(name),
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
    fn round_trips_an_override() {
        let target = PrimId([2; 16]);
        let key = override_key(id(), target, "material:base_color");
        assert_eq!(
            parse(&key),
            Some(Key::Override {
                site: id(),
                target,
                name: SmolStr::new("material:base_color"),
            })
        );
    }

    #[test]
    fn an_override_is_not_read_as_a_prim_property() {
        let key = override_key(id(), PrimId([2; 16]), "xform");
        assert!(!key.starts_with(PRIM_PREFIX));
    }

    #[test]
    fn a_bare_override_site_or_target_is_not_a_key() {
        assert_eq!(parse(&override_prefix(id())), None);
        assert_eq!(parse(&format!("o/{}/{}/", id(), PrimId([2; 16]))), None);
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
    fn nested_slot_spelling_is_rejected() {
        assert_eq!(parse(&format!("p/{}/mesh/POSITION/", id())), None);
    }

    #[test]
    fn parent_is_a_property_not_the_prim_key() {
        let parent_key = parent(id());
        assert!(parent_key.starts_with(&prim_prefix(id())));
        assert_ne!(parent_key, prim_prefix(id()));
    }
}
