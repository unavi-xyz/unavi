//! [`PinRecord`]: a document author's signed word on whether it is pinned.

use hsd::id::DocId;
use iroh_docs::{
    NamespaceId,
    NamespaceSecret,
};
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::signed::{
    NamespaceSigner,
    Signable,
    Signed,
};

use crate::{
    membership::SpaceId,
    replication::{
        clock,
        message::Rejected,
    },
};

/// Whether `doc` is pinned into `space`, as of `at`. Signed with the
/// document's namespace key, so any peer checks it against the document id.
#[derive(Serialize, Deserialize, Clone)]
pub struct PinRecord {
    pub doc:    DocId,
    pub space:  SpaceId,
    /// Microseconds, as the signing device's clock reads. Orders the author's
    /// records for a document.
    pub at:     u64,
    pub pinned: bool,
}

impl Signable for PinRecord {
    const SIGNING_CONTEXT: &'static str = "wired/space/pin";
}

/// Why a record was not accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RecordError {
    #[error("record names another document")]
    WrongDoc,
    #[error("record names another space")]
    WrongSpace,
    #[error("record is not signed by the document's namespace")]
    Forged,
}

/// A signed record with its payload decoded once.
#[derive(Clone)]
pub struct Record {
    pub signed:  Signed<PinRecord>,
    pub payload: PinRecord,
    /// The stamp ordering uses. The signed `at` cannot be rewritten, so a
    /// received record's is clamped here and `signed` relays the original.
    pub stamp:   u64,
}

impl Record {
    /// Signs `payload` with the namespace write key of the document it names.
    pub fn sign(key: &NamespaceSecret, payload: PinRecord) -> anyhow::Result<Self> {
        let signed = payload.sign(&NamespaceSigner(key))?;
        Ok(Self {
            stamp: payload.at,
            signed,
            payload,
        })
    }

    /// Decodes a record received at `recv` (micros), clamping its stamp to
    /// no later than the allowed skew.
    pub fn decode(signed: Signed<PinRecord>, recv: u64) -> Result<Self, Rejected> {
        let payload = signed.payload().map_err(|_| Rejected::Malformed)?;
        Ok(Self {
            stamp: clock::clamp_to(payload.at, recv),
            signed,
            payload,
        })
    }

    /// Checks that the record names `doc` in `space` and that `doc`'s
    /// namespace signed it.
    pub fn verify(&self, doc: DocId, space: SpaceId) -> Result<(), RecordError> {
        if self.payload.doc != doc {
            return Err(RecordError::WrongDoc);
        }
        if self.payload.space != space {
            return Err(RecordError::WrongSpace);
        }
        self.signed
            .verify_namespace(NamespaceId::from(&doc.0))
            .map_err(|_| RecordError::Forged)
    }

    /// Whether this record outranks `other` for the same document: the later
    /// one, and a withdrawal over a pin at the same instant.
    #[must_use]
    pub const fn supersedes(&self, other: &Self) -> bool {
        self.stamp > other.stamp
            || (self.stamp == other.stamp && !self.payload.pinned && other.payload.pinned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(seed: u8) -> NamespaceSecret {
        NamespaceSecret::from_bytes(&[seed; 32])
    }

    fn space(seed: u8) -> SpaceId {
        SpaceId([seed; 32])
    }

    fn record(key: &NamespaceSecret, space: SpaceId, at: u64, pinned: bool) -> Record {
        let payload = PinRecord {
            doc: DocId(*key.id().as_bytes()),
            space,
            at,
            pinned,
        };
        Record::sign(key, payload).expect("sign")
    }

    #[test]
    fn a_record_verifies_for_its_document_and_space_only() {
        let (ns, other) = (key(1), key(2));
        let rec = record(&ns, space(1), 5, true);
        let doc = DocId(*ns.id().as_bytes());

        assert_eq!(rec.verify(doc, space(1)), Ok(()));
        assert_eq!(
            rec.verify(DocId(*other.id().as_bytes()), space(1)),
            Err(RecordError::WrongDoc)
        );
        assert_eq!(rec.verify(doc, space(2)), Err(RecordError::WrongSpace));

        let mut forged = record(&other, space(1), 5, true);
        forged.payload.doc = doc;
        assert_eq!(
            forged.verify(doc, space(1)),
            Err(RecordError::Forged),
            "another namespace's key does not speak for this document"
        );
    }

    #[test]
    fn the_later_record_wins_and_a_withdrawal_wins_a_tie() {
        let ns = key(1);
        let pin = record(&ns, space(1), 5, true);
        let later = record(&ns, space(1), 6, true);
        let tie = record(&ns, space(1), 5, false);

        assert!(later.supersedes(&pin));
        assert!(!pin.supersedes(&later));
        assert!(tie.supersedes(&pin));
        assert!(!pin.supersedes(&tie));
        assert!(!pin.supersedes(&pin));
    }

    #[test]
    fn a_received_stamp_is_clamped_and_the_signed_one_kept() {
        let ns = key(1);
        let recv = 1_000_000_000;
        let far = record(&ns, space(1), u64::MAX, false);

        let received = Record::decode(far.signed, recv).expect("decode");

        assert_eq!(received.stamp, recv + clock::MAX_SKEW_MICROS);
        assert_eq!(received.payload.at, u64::MAX, "the signed stamp is relayed");
    }
}
