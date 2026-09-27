use std::{
    fmt::{
        Debug,
        Display,
        Formatter,
    },
    str::FromStr,
};

use smol_str::SmolStr;
use thiserror::Error;

use crate::bounds::MAX_NAME_BYTES;

/// Builds a `const` [`PropName`], failing the build on an invalid path.
///
/// Accepts a literal or a const expression, e.g. `concatcp!(GROUP, "/field")`.
#[macro_export]
macro_rules! prop_name {
    ($path:literal) => {
        $crate::property::name::PropName::from_static($path)
            .expect(concat!("invalid property name ", $path))
    };
    ($path:expr) => {
        $crate::property::name::PropName::from_static($path).expect("invalid property name")
    };
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PropNameError {
    #[error("property name {0:?} is not `<group>` or `<group>/<field>`")]
    Shape(SmolStr),
    #[error("property name is {0} bytes, over the cap of {MAX_NAME_BYTES}")]
    TooLong(usize),
}

/// A property's address within its prim.
/// Either `<group>` or `<group>/<field>`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropName {
    path:  SmolStr,
    /// The end of the group.
    split: usize,
}

impl PropName {
    #[must_use]
    pub const fn from_static(path: &'static str) -> Option<Self> {
        match parse_split(path.as_bytes()) {
            Some(split) => Some(Self {
                path: SmolStr::new_static(path),
                split,
            }),
            None => None,
        }
    }

    pub fn new(group: &str, field: &str) -> Result<Self, PropNameError> {
        format!("{group}/{field}").parse()
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn group(&self) -> &str {
        &self.path[..self.split]
    }

    /// `None` for a bare name.
    #[must_use]
    pub fn field(&self) -> Option<&str> {
        self.path.get(self.split + 1..)
    }
}

impl FromStr for PropName {
    type Err = PropNameError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let split = parse_split(path.as_bytes()).ok_or_else(|| {
            if path.len() > MAX_NAME_BYTES {
                PropNameError::TooLong(path.len())
            } else {
                PropNameError::Shape(SmolStr::new(path))
            }
        })?;
        Ok(Self {
            path: SmolStr::new(path),
            split,
        })
    }
}

const fn parse_split(path: &[u8]) -> Option<usize> {
    if path.len() > MAX_NAME_BYTES {
        return None;
    }
    let mut split = None;
    let mut i = 0;
    while i < path.len() {
        if path[i] == b'/' {
            if split.is_some() {
                return None;
            }
            split = Some(i);
        }
        i += 1;
    }
    match split {
        Some(at) if at > 0 && at + 1 < path.len() => Some(at),
        None if !path.is_empty() => Some(path.len()),
        _ => None,
    }
}

impl Display for PropName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path)
    }
}

impl Debug for PropName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PropName({:?})", self.path)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn splits_namespace_and_field() {
        let name = "material/binding".parse::<PropName>().expect("valid");
        assert_eq!(name.group(), "material");
        assert_eq!(name.field(), Some("binding"));
    }

    #[test]
    fn a_bare_name_is_all_namespace() {
        let name = "xform".parse::<PropName>().expect("valid");
        assert_eq!(name.group(), "xform");
        assert_eq!(name.field(), None);
    }

    #[rstest]
    #[case("")]
    #[case("/")]
    #[case("/binding")]
    #[case("material/")]
    #[case("ref/layer/x")]
    fn rejects_anything_but_one_or_two_segments(#[case] path: &str) {
        assert!(path.parse::<PropName>().is_err());
    }

    #[test]
    fn rejects_an_oversized_name() {
        let path = format!("a/{}", "b".repeat(MAX_NAME_BYTES));
        assert_eq!(
            path.parse::<PropName>(),
            Err(PropNameError::TooLong(path.len()))
        );
    }
}
