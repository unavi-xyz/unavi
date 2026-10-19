//! What the visor and the halo say to each other about lenses.
//!
//! Both are scripts on the same peer, talking over `wired:event` on global
//! channels with postcard payloads. This crate holds the names and the shapes
//! so neither side has to agree with the other by copying.

pub mod intake;
pub mod verdicts;

use serde::{
    Deserialize,
    Serialize,
};

/// A document, as `wired:core/ids.document-id` names it.
pub type DocumentId = (u64, u64, u64, u64);

/// The id of the travel lens.
pub const TRAVEL: &str = "travel";

/// Visor to everyone: every lens it has, as a [`Vec<Lens>`]. Sent at start,
/// after every change, and in answer to [`ASK`].
pub const LENSES: &str = "unavi:visor/lenses";

/// Halo to visor: please announce on [`LENSES`]. The payload is empty.
pub const ASK: &str = "unavi:visor/ask";

/// Halo to visor: put a lens on or take it off, as a [`Wear`].
pub const WEAR: &str = "unavi:visor/wear";

/// One layer of the visor, and whether it is on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lens {
    /// Stable across announcements; what a [`Wear`] names.
    pub id:          String,
    pub name:        String,
    pub description: String,
    pub worn:        bool,
}

/// A request to put a lens on or take it off.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wear {
    pub id:   String,
    pub worn: bool,
}

/// Whether the local user authors the document a script has loaded. `open` is
/// the document, `None` when it is not loaded, and `is_author` asks the host;
/// either being unable to say leaves the answer undecided.
pub fn authored<D>(open: Option<D>, is_author: impl FnOnce(&D) -> Option<bool>) -> Option<bool> {
    is_author(&open?)
}

pub fn encode_lenses(lenses: &[Lens]) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec(lenses)
}

pub fn decode_lenses(bytes: &[u8]) -> Result<Vec<Lens>, postcard::Error> {
    postcard::from_bytes(bytes)
}

pub fn encode_wear(wear: &Wear) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec(wear)
}

pub fn decode_wear(bytes: &[u8]) -> Result<Wear, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lens(id: &str, worn: bool) -> Lens {
        Lens {
            id: id.to_owned(),
            name: id.to_uppercase(),
            description: format!("the {id} lens"),
            worn,
        }
    }

    #[test]
    fn lenses_round_trip() {
        let lenses = vec![lens("travel", true), lens("build", false)];
        let bytes = encode_lenses(&lenses).expect("encodes");
        assert_eq!(decode_lenses(&bytes).expect("decodes"), lenses);
    }

    #[test]
    fn no_lenses_round_trip() {
        let bytes = encode_lenses(&[]).expect("encodes");
        assert_eq!(decode_lenses(&bytes).expect("decodes"), Vec::new());
    }

    #[test]
    fn wear_round_trips() {
        let wear = Wear {
            id:   "travel".to_owned(),
            worn: false,
        };
        let bytes = encode_wear(&wear).expect("encodes");
        assert_eq!(decode_wear(&bytes).expect("decodes"), wear);
    }

    #[test]
    fn an_unloaded_document_is_undecided() {
        assert_eq!(authored(None::<u8>, |_| Some(true)), None);
        assert_eq!(authored(Some(1_u8), |_| Some(false)), Some(false));
    }

    #[test]
    fn garbage_is_refused() {
        assert!(decode_wear(&[0xFF, 0xFF, 0xFF]).is_err());
        assert!(decode_lenses(&[0x05]).is_err());
    }
}
