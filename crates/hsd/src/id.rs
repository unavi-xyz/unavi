use std::{
    fmt::{
        Debug,
        Display,
        Formatter,
    },
    str::FromStr,
};

use base64::{
    Engine as _,
    display::Base64Display,
    engine::general_purpose::URL_SAFE_NO_PAD,
};
use rand::Rng;
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

pub const PRIM_ID_BYTES: usize = 16;
pub const PRIM_ID_CHARS: usize = 22;

#[derive(Error, Debug)]
pub enum IdError {
    #[error("expected {PRIM_ID_CHARS} base64url characters, got {0}")]
    Length(usize),
    #[error(transparent)]
    Parse(#[from] base64::DecodeSliceError),
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PrimId(pub [u8; PRIM_ID_BYTES]);

impl PrimId {
    #[must_use]
    pub fn new() -> Self {
        let mut bytes = [0u8; PRIM_ID_BYTES];
        rand::rng().fill(&mut bytes);
        Self(bytes)
    }

    /// Truncates 32 derived bytes into an id, for build-time ids that must be
    /// identical on every peer rather than random.
    #[must_use]
    pub fn from_digest(digest: &[u8; 32]) -> Self {
        let mut bytes = [0u8; PRIM_ID_BYTES];
        bytes.copy_from_slice(&digest[..PRIM_ID_BYTES]);
        Self(bytes)
    }
}

impl Display for PrimId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Writes the base64 straight into the formatter; `encode` would build
        // a 22-byte `String` to hand back.
        write!(f, "{}", Base64Display::new(&self.0, &URL_SAFE_NO_PAD))
    }
}

impl Debug for PrimId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrimId({self})")
    }
}

impl FromStr for PrimId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != PRIM_ID_CHARS {
            return Err(IdError::Length(s.len()));
        }
        let mut bytes = [0u8; PRIM_ID_BYTES];
        let written = URL_SAFE_NO_PAD.decode_slice(s, &mut bytes)?;
        if written != PRIM_ID_BYTES {
            return Err(IdError::Length(written));
        }
        Ok(Self(bytes))
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocId(pub [u8; 32]);

impl DocId {
    /// Derived ID for a realized reference site: the child document a prim
    /// stands for, so every peer computes the same ID.
    ///
    /// Derived from the parent document and the site prim rather than being
    /// the target's id, because two prims may reference one document and
    /// everything keyed by document id is per site.
    #[must_use]
    pub fn site(parent: Self, prim: PrimId) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"hsd:instance");
        hasher.update(&parent.0);
        hasher.update(&prim.0);
        Self(*hasher.finalize().as_bytes())
    }
}

impl Display for DocId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", blake3::Hash::from_bytes(self.0).to_hex())
    }
}

impl Debug for DocId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocId({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prim_id_round_trips() {
        for _ in 0..1000 {
            let id = PrimId::new();
            let s = id.to_string();
            assert_eq!(s.len(), PRIM_ID_CHARS);
            assert_eq!(s.parse::<PrimId>().expect("parse"), id);
        }
    }

    #[test]
    fn max_value_round_trips() {
        let id = PrimId([0xFF; PRIM_ID_BYTES]);
        assert_eq!(id.to_string().parse::<PrimId>().expect("parse"), id);
    }

    #[test]
    fn invalid_encoding_rejected() {
        let junk = "!".repeat(PRIM_ID_CHARS);
        assert!(junk.parse::<PrimId>().is_err());
    }

    #[test]
    fn wrong_length_rejected() {
        assert!("ABC".parse::<PrimId>().is_err());
    }

    #[test]
    fn site_id_is_deterministic() {
        let parent = DocId([7; 32]);
        let prim = PrimId([3; PRIM_ID_BYTES]);
        assert_eq!(DocId::site(parent, prim), DocId::site(parent, prim));
        assert_ne!(DocId::site(parent, prim), DocId::site(DocId([8; 32]), prim));
    }
}
