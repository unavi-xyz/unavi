//! Document key layout.
//!
//! iroh-docs applies Willow prefix semantics: an entry removes all of its
//! author's older entries under its own key as a prefix.
//!
//! Two rules, and both are enforced here rather than by convention:
//!
//! 1. No data lives at a key that prefixes another key. A property either holds
//!    its value at `p/<prim>/<name>/` or owns the namespace below it, never
//!    both.
//! 2. Every key ends with `/`, so `mesh:ref/` cannot prefix `mesh:reference/`.

use smol_str::SmolStr;

use crate::id::PrimId;

pub const META: &str = "meta/";
pub const PRIM_PREFIX: &str = "p/";

/// Every top-level prefix a reader has to ask for to hold a whole document.
pub const PREFIXES: [&str; 2] = [META, PRIM_PREFIX];

#[must_use]
pub fn prim_prefix(prim: PrimId) -> String {
    format!("{PRIM_PREFIX}{prim}/")
}

/// A property's key: `p/<prim>/<name>/`.
#[must_use]
pub fn prop(prim: PrimId, name: &str) -> String {
    format!("{PRIM_PREFIX}{prim}/{name}/")
}

/// A key inside the namespace a property owns: `p/<prim>/<name>/<tail>/`.
#[must_use]
pub fn prop_sub(prim: PrimId, name: &str, tail: &str) -> String {
    format!("{PRIM_PREFIX}{prim}/{name}/{tail}/")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Meta,
    Prop {
        prim: PrimId,
        name: SmolStr,
    },
    PropSub {
        prim: PrimId,
        name: SmolStr,
        tail: SmolStr,
    },
}

/// Parses a document key, returning `None` for anything this format does not
/// define. Unrecognized keys are ignored rather than rejected.
#[must_use]
pub fn parse(key: &str) -> Option<Key> {
    if key == META {
        return Some(Key::Meta);
    }

    let rest = key.strip_prefix(PRIM_PREFIX)?;
    let rest = rest.strip_suffix('/')?;
    let (prim, rest) = rest.split_once('/')?;
    let prim = prim.parse::<PrimId>().ok()?;

    match rest.split_once('/') {
        None => {
            if !is_valid_name(rest) {
                return None;
            }
            Some(Key::Prop {
                prim,
                name: SmolStr::new(rest),
            })
        }
        Some((name, tail)) => {
            if !is_valid_name(name) || tail.is_empty() {
                return None;
            }
            Some(Key::PropSub {
                prim,
                name: SmolStr::new(name),
                tail: SmolStr::new(tail),
            })
        }
    }
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
    fn round_trips_a_sub_key() {
        let key = prop_sub(id(), "ref", "target");
        assert_eq!(
            parse(&key),
            Some(Key::PropSub {
                prim: id(),
                name: SmolStr::new("ref"),
                tail: SmolStr::new("target"),
            })
        );
    }

    /// How a property divides its own namespace never reaches this module.
    #[test]
    fn a_tail_keeps_its_slashes() {
        let key = prop_sub(id(), "ref", "layer/abc/xform");
        assert_eq!(
            parse(&key),
            Some(Key::PropSub {
                prim: id(),
                name: SmolStr::new("ref"),
                tail: SmolStr::new("layer/abc/xform"),
            })
        );
    }

    #[test]
    fn a_name_sharing_another_names_prefix_is_still_its_own_property() {
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
}
