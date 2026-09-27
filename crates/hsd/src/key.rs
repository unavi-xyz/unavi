use std::fmt::{
    Display,
    Formatter,
};

use smol_str::SmolStr;

use crate::{
    id::PrimId,
    property::name::PropName,
};

pub const META: &str = "meta/";
pub const PRIM_PREFIX: &str = "p/";

/// Every top-level prefix a reader has to ask for to hold a whole document.
pub const PREFIXES: [&str; 2] = [META, PRIM_PREFIX];

/// The prefix every key of `prim` lies under.
#[must_use]
pub fn prim_prefix(prim: PrimId) -> String {
    format!("{PRIM_PREFIX}{prim}/")
}

/// Where a key sits in a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Meta,
    /// A property at `p/<prim>/<name>/`.
    Prop {
        prim: PrimId,
        name: PropName,
    },
    /// Three or more segments below the prim. `tail` keeps its slashes.
    Nested {
        prim:  PrimId,
        group: SmolStr,
        tail:  SmolStr,
    },
}

impl Key {
    #[must_use]
    pub fn prop(prim: PrimId, name: &PropName) -> Self {
        Self::Prop {
            prim,
            name: name.clone(),
        }
    }

    /// A key deeper than a property, below `group`.
    #[must_use]
    pub fn nested(prim: PrimId, group: &str, tail: &str) -> Self {
        Self::Nested {
            prim,
            group: SmolStr::new(group),
            tail: SmolStr::new(tail),
        }
    }

    /// Unrecognized keys return `None`.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        if key == META {
            return Some(Self::Meta);
        }

        let rest = key.strip_prefix(PRIM_PREFIX)?;
        let rest = rest.strip_suffix('/')?;
        let (prim, rest) = rest.split_once('/')?;
        let prim = prim.parse::<PrimId>().ok()?;

        if let Some((group, tail)) = rest.split_once('/')
            && tail.contains('/')
        {
            if group.is_empty() || tail.split('/').any(str::is_empty) {
                return None;
            }
            return Some(Self::Nested {
                prim,
                group: SmolStr::new(group),
                tail: SmolStr::new(tail),
            });
        }
        Some(Self::Prop {
            prim,
            name: rest.parse().ok()?,
        })
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Meta => f.write_str(META),
            Self::Prop { prim, name } => write!(f, "{PRIM_PREFIX}{prim}/{name}/"),
            Self::Nested { prim, group, tail } => write!(f, "{PRIM_PREFIX}{prim}/{group}/{tail}/"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> PrimId {
        PrimId([1; 16])
    }

    fn name(path: &str) -> PropName {
        path.parse().expect("valid")
    }

    #[test]
    fn round_trips_a_property() {
        let key = Key::prop(id(), &name("material/binding"));
        assert_eq!(
            Key::parse(&key.to_string()),
            Some(Key::Prop {
                prim: id(),
                name: name("material/binding"),
            })
        );
    }

    #[test]
    fn a_nested_tail_keeps_its_slashes() {
        let key = Key::nested(id(), "ref", "layer/abc/xform");
        assert_eq!(
            Key::parse(&key.to_string()),
            Some(Key::Nested {
                prim:  id(),
                group: SmolStr::new("ref"),
                tail:  SmolStr::new("layer/abc/xform"),
            })
        );
    }

    #[test]
    fn a_property_lies_under_its_namespace() {
        let name = name("material/binding");
        let prefix = format!("{}{}/", prim_prefix(id()), name.group());
        assert!(Key::prop(id(), &name).to_string().starts_with(&prefix));
    }

    #[test]
    fn a_field_sharing_another_fields_prefix_is_still_its_own_property() {
        assert_eq!(
            Key::parse(&format!("{PRIM_PREFIX}{}/material/binding_extra/", id())),
            Some(Key::Prop {
                prim: id(),
                name: name("material/binding_extra"),
            })
        );
    }

    #[test]
    fn round_trips_meta() {
        assert_eq!(Key::parse(META), Some(Key::Meta));
    }

    #[test]
    fn a_bare_property_is_its_namespace_prefix() {
        let name = name("xform");
        assert_eq!(
            Key::prop(id(), &name).to_string(),
            format!("{}{}/", prim_prefix(id()), "xform")
        );
        assert_eq!(
            Key::parse(&Key::prop(id(), &name).to_string()),
            Some(Key::Prop { prim: id(), name })
        );
    }

    #[test]
    fn a_prim_alone_is_not_a_key() {
        assert_eq!(Key::parse(&prim_prefix(id())), None);
    }

    #[test]
    fn missing_trailing_slash_is_rejected() {
        assert_eq!(Key::parse(&format!("p/{}/xform", id())), None);
    }
}
