use std::{
    fmt,
    str::FromStr,
};

use rand::Rng;
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;
use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

pub const PRIM_ID_BYTES: usize = 16;
pub const PRIM_ID_CHARS: usize = 26;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[derive(Error, Debug)]
pub enum IdError {
    #[error("expected {PRIM_ID_CHARS} characters, got {0}")]
    Length(usize),
    #[error("invalid base32 character {0:?}")]
    Character(char),
    #[error("value overflows 128 bits")]
    Overflow,
}

/// A ULID: 48 bits of millisecond timestamp followed by 80 random bits,
/// rendered as 26 characters of Crockford base32.
///
/// Fixed length, so no id is a prefix of another.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PrimId(pub [u8; PRIM_ID_BYTES]);

impl PrimId {
    #[must_use]
    pub fn new() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as u64)
            & 0x0000_FFFF_FFFF_FFFF;
        let random: u128 = rand::rng().random::<u128>() & ((1 << 80) - 1);
        Self((u128::from(millis) << 80 | random).to_be_bytes())
    }

    /// Truncates 32 derived bytes into an id, for build-time ids that must be
    /// identical on every peer rather than time-ordered.
    #[must_use]
    pub fn from_digest(digest: &[u8; 32]) -> Self {
        let mut bytes = [0u8; PRIM_ID_BYTES];
        bytes.copy_from_slice(&digest[..PRIM_ID_BYTES]);
        Self(bytes)
    }
}

impl fmt::Display for PrimId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut value = u128::from_be_bytes(self.0);
        let mut out = [0u8; PRIM_ID_CHARS];
        for slot in out.iter_mut().rev() {
            *slot = ALPHABET[(value & 0x1F) as usize];
            value >>= 5;
        }
        f.write_str(std::str::from_utf8(&out).map_err(|_| fmt::Error)?)
    }
}

impl fmt::Debug for PrimId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrimId({self})")
    }
}

impl FromStr for PrimId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != PRIM_ID_CHARS {
            return Err(IdError::Length(s.len()));
        }
        let mut value: u128 = 0;
        for c in s.chars() {
            let digit = decode_char(c)?;
            value = value.checked_mul(32).ok_or(IdError::Overflow)?;
            value = value
                .checked_add(u128::from(digit))
                .ok_or(IdError::Overflow)?;
        }
        Ok(Self(value.to_be_bytes()))
    }
}

const fn decode_char(c: char) -> Result<u8, IdError> {
    let upper = c.to_ascii_uppercase();
    match upper {
        '0' | 'O' => Ok(0),
        '1' | 'I' | 'L' => Ok(1),
        '2'..='9' => Ok(upper as u8 - b'0'),
        'A'..='H' => Ok(upper as u8 - b'A' + 10),
        'J' | 'K' => Ok(upper as u8 - b'J' + 18),
        'M' | 'N' => Ok(upper as u8 - b'M' + 20),
        'P'..='T' => Ok(upper as u8 - b'P' + 22),
        'V'..='Z' => Ok(upper as u8 - b'V' + 27),
        _ => Err(IdError::Character(c)),
    }
}

/// Identifies an HSD document. For a shared document it is the
/// [`NamespaceId`](iroh_docs::NamespaceId); for a prefab instance it is derived
/// so every peer computes the same id for the same instance.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocId(pub [u8; 32]);

impl DocId {
    #[must_use]
    pub fn instance(parent: Self, prim: PrimId) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"hsd:instance");
        hasher.update(&parent.0);
        hasher.update(&prim.0);
        Self(*hasher.finalize().as_bytes())
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", blake3::Hash::from_bytes(self.0).to_hex())
    }
}

impl fmt::Debug for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocId({self})")
    }
}

/// A blake3 content hash, as carried by every entry value.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlobId(pub [u8; 32]);

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", blake3::Hash::from_bytes(self.0).to_hex())
    }
}

impl fmt::Debug for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlobId({self})")
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
    fn prim_ids_are_time_ordered() {
        let a = PrimId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = PrimId::new();
        assert!(a < b);
        assert!(a.to_string() < b.to_string());
    }

    #[test]
    fn no_prim_id_prefixes_another() {
        let a = PrimId::new().to_string();
        let b = PrimId::from_digest(&[0xAB; 32]).to_string();
        assert_eq!(a.len(), b.len());
        assert!(!a.starts_with(&b) || a == b);
    }

    #[test]
    fn max_value_round_trips() {
        let id = PrimId([0xFF; PRIM_ID_BYTES]);
        assert_eq!(id.to_string().parse::<PrimId>().expect("parse"), id);
    }

    #[test]
    fn crockford_ambiguities_decode() {
        let canonical = PrimId([0; PRIM_ID_BYTES]).to_string();
        assert_eq!(canonical, "0".repeat(PRIM_ID_CHARS));
        assert_eq!(
            "O".repeat(PRIM_ID_CHARS).parse::<PrimId>().expect("parse"),
            PrimId([0; PRIM_ID_BYTES])
        );
    }

    #[test]
    fn wrong_length_rejected() {
        assert!("ABC".parse::<PrimId>().is_err());
    }

    #[test]
    fn instance_id_is_deterministic() {
        let parent = DocId([7; 32]);
        let prim = PrimId([3; PRIM_ID_BYTES]);
        assert_eq!(DocId::instance(parent, prim), DocId::instance(parent, prim));
        assert_ne!(
            DocId::instance(parent, prim),
            DocId::instance(DocId([8; 32]), prim)
        );
    }
}
