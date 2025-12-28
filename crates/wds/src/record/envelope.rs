use loro::{ExportMode, LoroDoc, VersionVector};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

use crate::signed_bytes::Signable;

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    author: Did,
    from: VersionVector,
    to: VersionVector,
    ops: Vec<u8>,
}

impl Signable for Envelope {}

impl Envelope {
    /// # Errors
    ///
    /// Returns an error if the document could not be exported.
    pub fn updates(author: Did, doc: &LoroDoc, from: VersionVector) -> anyhow::Result<Self> {
        let to = doc.oplog_vv();
        let ops = doc.export(ExportMode::updates(&from))?;
        Ok(Self {
            author,
            from,
            to,
            ops,
        })
    }

    /// # Errors
    ///
    /// Returns an error if the document could not be exported.
    pub fn all_updates(author: Did, doc: &LoroDoc) -> anyhow::Result<Self> {
        Self::updates(author, doc, VersionVector::new())
    }

    #[must_use]
    pub const fn author(&self) -> &Did {
        &self.author
    }

    #[must_use]
    pub const fn start_vv(&self) -> &VersionVector {
        &self.from
    }

    #[must_use]
    pub const fn end_vv(&self) -> &VersionVector {
        &self.to
    }

    #[must_use]
    pub fn ops(&self) -> &[u8] {
        &self.ops
    }
}
