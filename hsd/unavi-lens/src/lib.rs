//! What the visor and the halo say to each other about lenses.
//!
//! Both are scripts on the same peer, talking over `wired:event` on global
//! channels with postcard payloads. This crate holds the names and the shapes
//! so neither side has to agree with the other by copying.
//!
//! # Wearing
//!
//! [`WEAR`] sets the user's own choice of a lens, which only the user's
//! requests change.
//!
//! # Holding
//!
//! A tool sends [`HOLD`] with a [`Hold`] when it takes up or lays down a lens,
//! such as an inspector that needs document outlines while it is in hand. The
//! visor keeps one holder per sending document and lens, and shows a lens
//! while it is worn or any holder holds it. A hold never changes what is worn,
//! so letting go leaves the lens as the user had it. The visor acts only on
//! holds from documents the local user authors.

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
/// The id of the social lens.
pub const SOCIAL: &str = "social";
/// The id of the documents lens.
pub const DOCUMENTS: &str = "documents";

/// Visor to everyone: every lens it has, as a [`Vec<Lens>`]. Sent at start,
/// after every change, and in answer to [`ASK`].
pub const LENSES: &str = "unavi:visor/lenses";

/// Halo to visor: please announce on [`LENSES`]. The payload is empty.
pub const ASK: &str = "unavi:visor/ask";

/// Halo to visor: put a lens on or take it off, as a [`Wear`].
pub const WEAR: &str = "unavi:visor/wear";

/// Any tool to visor: the sender holds a lens or lets it go, as a [`Hold`].
pub const HOLD: &str = "unavi:visor/hold";

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

/// A tool taking up or laying down a lens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hold {
    /// The id of the lens, as in [`Lens::id`].
    pub lens: String,
    pub held: bool,
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

pub fn encode_hold(hold: &Hold) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec(hold)
}

pub fn decode_hold(bytes: &[u8]) -> Result<Hold, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_round_trips() {
        let hold = Hold {
            lens: DOCUMENTS.to_owned(),
            held: true,
        };
        let bytes = encode_hold(&hold).expect("encodes");
        assert_eq!(decode_hold(&bytes).expect("decodes"), hold);
    }

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
